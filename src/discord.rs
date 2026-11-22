use crate::pack::{Message, format_timestamp, make_preview};
use crate::store::{BatchModeSettings, EditBeforeDeleteSettings, EditContentMode};
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

const RANDOM_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

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
    Edited {
        msg_id: String,
    },
    EditFailed {
        msg_id: String,
        status: String,
        reason: String,
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

#[derive(Debug, Clone)]
pub enum DmFetchEvent {
    Progress {
        scanned: usize,
        found: usize,
    },
    Done {
        user: ValidatedUser,
        channel_id: String,
        channel_name: String,
        messages: Vec<Message>,
        cancelled: bool,
    },
    Error(String),
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

#[derive(Deserialize)]
struct ApiRecipient {
    id: Option<String>,
    username: Option<String>,
    global_name: Option<String>,
}

#[derive(Deserialize)]
struct ApiChannel {
    id: String,
    name: Option<String>,
    recipients: Option<Vec<ApiRecipient>>,
}

#[derive(Deserialize)]
struct ApiAuthor {
    id: String,
}

#[derive(Deserialize)]
struct ApiMessage {
    id: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    timestamp: String,
    #[serde(rename = "type", default)]
    kind: u8,
    author: ApiAuthor,
}

pub fn extract_snowflake(input: &str) -> Option<String> {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let segment = trimmed.rsplit('/').next().unwrap_or(trimmed);
    let candidate = segment.split(['?', '#']).next().unwrap_or(segment);
    if !candidate.is_empty() && candidate.bytes().all(|b| b.is_ascii_digit()) {
        Some(candidate.to_string())
    } else {
        None
    }
}

pub fn build_replacement_content(cfg: &EditBeforeDeleteSettings) -> String {
    match cfg.mode {
        EditContentMode::Custom => {
            let trimmed = cfg.custom_text.trim();
            if trimmed.is_empty() {
                ".".to_string()
            } else {
                trimmed.chars().take(2000).collect()
            }
        }
        EditContentMode::Random => {
            let len = cfg.random_length.clamp(1, 2000) as usize;
            let mut rng = rand::rng();
            (0..len)
                .map(|_| RANDOM_CHARSET[rng.random_range(0..RANDOM_CHARSET.len())] as char)
                .collect()
        }
    }
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

async fn resolve_dm_channel(
    client: &reqwest::Client,
    headers: &HeaderMap,
    target_id: &str,
    self_id: &str,
) -> Result<(String, String), String> {
    let url = format!("https://discord.com/api/v10/channels/{target_id}");
    let resp = client
        .get(&url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let channel: ApiChannel = if resp.status().is_success() {
        resp.json().await.map_err(|e| e.to_string())?
    } else {
        let orig_code = resp.status().as_u16();
        let orig_reason = parse_error_reason(resp).await;

        // Fall back to opening a DM in case the user supplied a recipient User ID.
        let dm_resp = client
            .post("https://discord.com/api/v10/users/@me/channels")
            .headers(headers.clone())
            .json(&json!({ "recipient_id": target_id }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if dm_resp.status().is_success() {
            dm_resp.json().await.map_err(|e| e.to_string())?
        } else if orig_reason.is_empty() {
            return Err(format!("HTTP {orig_code}"));
        } else {
            return Err(format!("HTTP {orig_code}: {orig_reason}"));
        }
    };

    let display_name = channel
        .name
        .filter(|n| !n.is_empty())
        .or_else(|| {
            let names: Vec<String> = channel
                .recipients
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.id.as_deref() != Some(self_id))
                .filter_map(|r| {
                    r.global_name
                        .filter(|s| !s.is_empty())
                        .or(r.username.filter(|s| !s.is_empty()))
                })
                .collect();
            if names.is_empty() {
                None
            } else {
                Some(names.join(", "))
            }
        })
        .unwrap_or_else(|| channel.id.clone());

    Ok((channel.id, display_name))
}

pub async fn fetch_dm_messages(
    client: reqwest::Client,
    token: String,
    target_id: String,
    stop: Arc<StopSignal>,
    tx: mpsc::UnboundedSender<DmFetchEvent>,
    repaint: impl Fn() + Send + Sync + 'static,
) {
    let send = |ev: DmFetchEvent| {
        let _ = tx.send(ev);
        repaint();
    };

    let user = match validate_token(&client, &token).await {
        Ok(u) => u,
        Err(err) => {
            send(DmFetchEvent::Error(err));
            return;
        }
    };

    let headers = match build_headers(&token) {
        Ok(h) => h,
        Err(err) => {
            send(DmFetchEvent::Error(err));
            return;
        }
    };

    let (channel_id, channel_name) = match resolve_dm_channel(&client, &headers, &target_id, &user.id).await {
        Ok(res) => res,
        Err(err) => {
            send(DmFetchEvent::Error(err));
            return;
        }
    };

    let mut before: Option<String> = None;
    let mut scanned = 0usize;
    let mut messages: Vec<Message> = Vec::new();

    loop {
        if stop.is_stopped() {
            break;
        }

        let url = match &before {
            Some(id) => format!(
                "https://discord.com/api/v10/channels/{channel_id}/messages?limit=100&before={id}"
            ),
            None => format!("https://discord.com/api/v10/channels/{channel_id}/messages?limit=100"),
        };

        let resp = match client.get(&url).headers(headers.clone()).send().await {
            Ok(r) => r,
            Err(err) => {
                send(DmFetchEvent::Error(err.to_string()));
                return;
            }
        };

        let code = resp.status().as_u16();
        if code == 429 {
            let retry_secs = resp
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v.get("retry_after").and_then(|r| r.as_f64()))
                .unwrap_or(3.0);
            let wait_ms = (retry_secs * 1000.0).ceil().max(500.0) as u64 + 500;
            if stop.sleep_interruptible(Duration::from_millis(wait_ms)).await {
                break;
            }
            continue;
        }

        if !resp.status().is_success() {
            let reason = parse_error_reason(resp).await;
            let err = if reason.is_empty() {
                format!("HTTP {code}")
            } else {
                format!("HTTP {code}: {reason}")
            };
            send(DmFetchEvent::Error(err));
            return;
        }

        let batch: Vec<ApiMessage> = match resp.json().await {
            Ok(b) => b,
            Err(err) => {
                send(DmFetchEvent::Error(err.to_string()));
                return;
            }
        };

        let batch_len = batch.len();
        if batch_len == 0 {
            break;
        }

        let oldest_id = batch.last().map(|m| m.id.clone());
        scanned += batch_len;

        for m in batch {
            // Only normal, reply, and slash-command messages from self can be deleted.
            if m.author.id == user.id && matches!(m.kind, 0 | 19 | 20) {
                let formatted_time = format_timestamp(&m.timestamp);
                let preview = make_preview(&m.content);
                messages.push(Message {
                    id: m.id,
                    formatted_time,
                    content: m.content,
                    preview,
                    channel_id: channel_id.clone(),
                    is_dm: true,
                    recipient_ids: Vec::new(),
                    channel_name: channel_name.clone(),
                });
            }
        }

        send(DmFetchEvent::Progress {
            scanned,
            found: messages.len(),
        });

        if batch_len < 100 {
            break;
        }

        before = oldest_id;
        if stop.sleep_interruptible(Duration::from_millis(350)).await {
            break;
        }
    }

    send(DmFetchEvent::Done {
        user,
        channel_id,
        channel_name,
        messages,
        cancelled: stop.is_stopped(),
    });
}

pub struct DeletionJob {
    pub client: reqwest::Client,
    pub messages: Vec<Message>,
    pub token: String,
    pub delay_secs: u32,
    pub batch_mode: Option<BatchModeSettings>,
    pub edit_before_delete: Option<EditBeforeDeleteSettings>,
    pub history_set: HashSet<String>,
    pub stop: Arc<StopSignal>,
    pub tx: mpsc::UnboundedSender<DeletionEvent>,
}

async fn edit_message(
    client: &reqwest::Client,
    headers: &HeaderMap,
    url: &str,
    msg_id: &str,
    new_content: &str,
    stop: &StopSignal,
    send: &impl Fn(DeletionEvent),
) -> bool {
    let payload = json!({ "content": new_content });
    match client
        .patch(url)
        .headers(headers.clone())
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let code = resp.status().as_u16();
            if resp.status().is_success() {
                send(DeletionEvent::Edited {
                    msg_id: msg_id.to_string(),
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
                    return true;
                }

                match client
                    .patch(url)
                    .headers(headers.clone())
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(retry_resp) if retry_resp.status().is_success() => {
                        send(DeletionEvent::Edited {
                            msg_id: msg_id.to_string(),
                        });
                    }
                    Ok(retry_resp) => {
                        let retry_code = retry_resp.status().as_u16();
                        let reason = parse_error_reason(retry_resp).await;
                        send(DeletionEvent::EditFailed {
                            msg_id: msg_id.to_string(),
                            status: retry_code.to_string(),
                            reason,
                        });
                    }
                    Err(err) => {
                        send(DeletionEvent::EditFailed {
                            msg_id: msg_id.to_string(),
                            status: "exception".to_string(),
                            reason: err.to_string(),
                        });
                    }
                }
            } else {
                let reason = parse_error_reason(resp).await;
                send(DeletionEvent::EditFailed {
                    msg_id: msg_id.to_string(),
                    status: code.to_string(),
                    reason,
                });
            }
        }
        Err(err) => {
            send(DeletionEvent::EditFailed {
                msg_id: msg_id.to_string(),
                status: "exception".to_string(),
                reason: err.to_string(),
            });
        }
    }
    false
}

pub async fn run_deletion(job: DeletionJob, repaint: impl Fn() + Send + Sync + 'static) {
    let DeletionJob {
        client,
        messages,
        token,
        delay_secs,
        batch_mode,
        edit_before_delete,
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
    let edit_cfg = edit_before_delete.filter(|e| e.enabled);

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

            let url = format!(
                "https://discord.com/api/v10/channels/{channel_id}/messages/{}",
                msg.id
            );

            if let Some(cfg) = &edit_cfg {
                let new_content = build_replacement_content(cfg);
                if msg.content != new_content {
                    let stopped = edit_message(
                        &client,
                        &headers,
                        &url,
                        &msg.id,
                        &new_content,
                        &stop,
                        &send,
                    )
                    .await;
                    if stopped || stop.sleep_interruptible(delay).await {
                        break 'outer;
                    }
                }
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_snowflake_from_raw_id_and_url() {
        assert_eq!(
            extract_snowflake("  123456789012345678 "),
            Some("123456789012345678".to_string())
        );
        assert_eq!(
            extract_snowflake("https://discord.com/channels/@me/987654321098765432/"),
            Some("987654321098765432".to_string())
        );
        assert_eq!(extract_snowflake("not-an-id"), None);
        assert_eq!(extract_snowflake(""), None);
    }

    #[test]
    fn builds_custom_and_random_replacement_content() {
        let custom = EditBeforeDeleteSettings {
            enabled: true,
            mode: EditContentMode::Custom,
            custom_text: "overwritten".to_string(),
            random_length: 10,
        };
        assert_eq!(build_replacement_content(&custom), "overwritten");

        let blank = EditBeforeDeleteSettings {
            enabled: true,
            mode: EditContentMode::Custom,
            custom_text: "   ".to_string(),
            random_length: 10,
        };
        assert_eq!(build_replacement_content(&blank), ".");

        let random = EditBeforeDeleteSettings {
            enabled: true,
            mode: EditContentMode::Random,
            custom_text: String::new(),
            random_length: 24,
        };
        let generated = build_replacement_content(&random);
        assert_eq!(generated.len(), 24);
        assert!(generated.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
