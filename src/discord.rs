use crate::pack::Message;
use crate::store::BatchModeSettings;
use rand::Rng;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{Notify, mpsc};

// Standard browser User-Agent to avoid trivial client fingerprinting.
const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0";

#[derive(Debug, Clone)]
pub struct ValidatedUser {
    pub username: String,
    pub id: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeletionEvent {
    BatchStart {
        batch_index: usize,
        batch_total: usize,
        batch_size: usize,
    },
    Success {
        msg_id: String,
        deleted: usize,
        errors: usize,
    },
    Error {
        msg_id: String,
        deleted: usize,
        errors: usize,
        status: String,
        reason: String,
    },
    RateLimit {
        retry_after_ms: u64,
    },
    BatchBreak {
        break_ms: u64,
        batch_index: usize,
        batch_total: usize,
    },
    BatchBreakTick {
        remaining_ms: u64,
    },
    Stopped {
        deleted: usize,
        errors: usize,
    },
    Done {
        deleted: usize,
        errors: usize,
    },
}

pub struct StopSignal {
    stopped: AtomicBool,
    notify: Notify,
}

impl StopSignal {
    pub fn new() -> Self {
        Self {
            stopped: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    pub fn request_stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    async fn sleep_interruptible(&self, duration: Duration) -> bool {
        if self.is_stopped() {
            return true;
        }
        tokio::select! {
            _ = tokio::time::sleep(duration) => self.is_stopped(),
            _ = self.notify.notified() => true,
        }
    }
}

#[derive(Deserialize)]
struct DiscordUserResponse {
    username: String,
    id: String,
    avatar: Option<String>,
}

#[derive(Deserialize)]
struct DmChannelResponse {
    id: Option<String>,
}

fn build_headers(token: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(token).map_err(|e| e.to_string())?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static(BROWSER_USER_AGENT));
    Ok(headers)
}

pub async fn validate_token(client: &reqwest::Client, token: &str) -> Result<ValidatedUser, String> {
    let headers = build_headers(token)?;
    let resp = client
        .get("https://discord.com/api/v10/users/@me")
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().as_u16() == 401 {
        return Err("Invalid token".to_string());
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }

    let user: DiscordUserResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(ValidatedUser {
        username: user.username,
        id: user.id,
        avatar: user.avatar,
    })
}

pub async fn fetch_avatar_bytes(
    client: &reqwest::Client,
    user_id: &str,
    avatar_hash: &str,
) -> Option<Vec<u8>> {
    let url = format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=64");
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.bytes().await.ok().map(|b| b.to_vec())
}

async fn parse_error_reason(resp: reqwest::Response) -> String {
    let text = resp.text().await.unwrap_or_default();
    if let Ok(val) = serde_json::from_str::<Value>(&text) {
        if let Some(msg) = val.get("message").and_then(|m| m.as_str()) {
            return msg.to_string();
        }
        return val.to_string();
    }
    text
}

pub struct DeletionJob {
    pub client: reqwest::Client,
    pub messages: Vec<Message>,
    pub token: String,
    pub delay_secs: u32,
    pub batch_mode: Option<BatchModeSettings>,
    pub history_set: HashSet<String>,
    pub stop: Arc<StopSignal>,
    pub tx: mpsc::UnboundedSender<DeletionEvent>,
}

pub async fn run_deletion(job: DeletionJob, repaint: impl Fn() + Send + Sync + 'static) {
    let DeletionJob {
        client,
        messages,
        token,
        delay_secs,
        batch_mode,
        history_set,
        stop,
        tx,
    } = job;
    let send = |ev: DeletionEvent| {
        let _ = tx.send(ev);
        repaint();
    };

    let Ok(headers) = build_headers(&token) else {
        send(DeletionEvent::Done {
            deleted: 0,
            errors: messages.len(),
        });
        return;
    };

    let delay = Duration::from_secs(delay_secs.max(1) as u64);
    let batch_enabled = batch_mode.as_ref().is_some_and(|b| b.enabled);

    let batches: Vec<(Vec<Message>, u64)> = if let Some(bm) = batch_mode.filter(|b| b.enabled) {
        let min_batch = bm.min_batch.max(1) as usize;
        let max_batch = (bm.max_batch as usize).max(min_batch);
        let min_break_ms = (bm.min_break_min as u64) * 60_000;
        let max_break_ms = ((bm.max_break_min as u64) * 60_000).max(min_break_ms);
        let mut rng = rand::rng();
        let mut out = Vec::new();
        let mut idx = 0;
        while idx < messages.len() {
            let size = rng.random_range(min_batch..=max_batch);
            let end = (idx + size).min(messages.len());
            let break_ms = rng.random_range(min_break_ms..=max_break_ms);
            out.push((messages[idx..end].to_vec(), break_ms));
            idx = end;
        }
        out
    } else {
        vec![(messages, 0)]
    };

    let mut deleted = 0usize;
    let mut errors = 0usize;
    let batch_total = batches.len();

    'outer: for (batch_idx, (batch_msgs, break_ms)) in batches.into_iter().enumerate() {
        if batch_enabled {
            send(DeletionEvent::BatchStart {
                batch_index: batch_idx + 1,
                batch_total,
                batch_size: batch_msgs.len(),
            });
        }

        for msg in batch_msgs {
            if stop.is_stopped() {
                break 'outer;
            }

            if history_set.contains(&msg.id) {
                deleted += 1;
                send(DeletionEvent::Success {
                    msg_id: msg.id,
                    deleted,
                    errors,
                });
                continue;
            }

            let mut channel_id = msg.channel_id.clone();

            // Closed DMs must be reopened to ensure the channel ID is active.
            if msg.is_dm && !msg.recipient_ids.is_empty() {
                for recipient_id in &msg.recipient_ids {
                    let res = client
                        .post("https://discord.com/api/v10/users/@me/channels")
                        .headers(headers.clone())
                        .json(&json!({ "recipient_id": recipient_id }))
                        .send()
                        .await;
                    if let Ok(resp) = res
                        && resp.status().is_success()
                        && let Ok(dm) = resp.json::<DmChannelResponse>().await
                        && let Some(id) = dm.id
                    {
                        channel_id = id;
                        break;
                    }
                }
            }

            let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages/{}", msg.id);
            match client.delete(&url).headers(headers.clone()).send().await {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    if code == 204 || code == 200 {
                        deleted += 1;
                        send(DeletionEvent::Success {
                            msg_id: msg.id,
                            deleted,
                            errors,
                        });
                    } else if code == 429 {
                        let retry_secs = resp
                            .json::<Value>()
                            .await
                            .ok()
                            .and_then(|v| v.get("retry_after").and_then(|r| r.as_f64()))
                            .unwrap_or(5.0);
                        let retry_after_ms = (retry_secs * 1000.0).ceil().max(0.0) as u64;
                        send(DeletionEvent::RateLimit { retry_after_ms });

                        if stop
                            .sleep_interruptible(Duration::from_millis(retry_after_ms + 500))
                            .await
                        {
                            break 'outer;
                        }

                        match client.delete(&url).headers(headers.clone()).send().await {
                            Ok(retry_resp) => {
                                let retry_code = retry_resp.status().as_u16();
                                if retry_code == 204 || retry_code == 200 {
                                    deleted += 1;
                                    send(DeletionEvent::Success {
                                        msg_id: msg.id,
                                        deleted,
                                        errors,
                                    });
                                } else {
                                    let reason = parse_error_reason(retry_resp).await;
                                    errors += 1;
                                    send(DeletionEvent::Error {
                                        msg_id: msg.id,
                                        deleted,
                                        errors,
                                        status: retry_code.to_string(),
                                        reason,
                                    });
                                }
                            }
                            Err(err) => {
                                errors += 1;
                                send(DeletionEvent::Error {
                                    msg_id: msg.id,
                                    deleted,
                                    errors,
                                    status: "exception".to_string(),
                                    reason: err.to_string(),
                                });
                            }
                        }
                    } else {
                        let reason = parse_error_reason(resp).await;
                        errors += 1;
                        send(DeletionEvent::Error {
                            msg_id: msg.id,
                            deleted,
                            errors,
                            status: code.to_string(),
                            reason,
                        });
                    }
                }
                Err(err) => {
                    errors += 1;
                    send(DeletionEvent::Error {
                        msg_id: msg.id,
                        deleted,
                        errors,
                        status: "exception".to_string(),
                        reason: err.to_string(),
                    });
                }
            }

            if stop.sleep_interruptible(delay).await {
                break 'outer;
            }
        }

        if batch_enabled && batch_idx + 1 < batch_total {
            send(DeletionEvent::BatchBreak {
                break_ms,
                batch_index: batch_idx + 1,
                batch_total,
            });

            let mut remaining = break_ms;
            while remaining > 0 {
                if stop.is_stopped() {
                    break 'outer;
                }
                let wait = remaining.min(10_000);
                if stop.sleep_interruptible(Duration::from_millis(wait)).await {
                    break 'outer;
                }
                remaining = remaining.saturating_sub(wait);
                if remaining > 0 {
                    send(DeletionEvent::BatchBreakTick {
                        remaining_ms: remaining,
                    });
                }
            }
        }
    }

    if stop.is_stopped() {
        send(DeletionEvent::Stopped { deleted, errors });
    } else {
        send(DeletionEvent::Done { deleted, errors });
    }
}
