use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub formatted_time: String,
    pub content: String,
    pub preview: String,
    pub channel_id: String,
    pub is_dm: bool,
    pub recipient_ids: Vec<String>,
    pub channel_name: String,
}

#[derive(Debug, Clone)]
pub struct ChannelGroup {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct ServerGroup {
    pub name: String,
    pub channels: Vec<ChannelGroup>,
}

#[derive(Debug, Clone)]
pub struct DmGroup {
    pub name: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct PackData {
    pub servers: Vec<ServerGroup>,
    pub dms: Vec<DmGroup>,
    pub total_messages: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupType {
    Server,
    Dm,
}

#[derive(Debug, Clone)]
pub struct FilteredMessage {
    pub msg: Message,
    pub group_name: String,
    pub group_type: GroupType,
}

#[derive(Deserialize)]
struct RawGuild {
    id: Option<Value>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct RawChannelInfo {
    id: Option<Value>,
    #[serde(rename = "type")]
    kind: Option<Value>,
    name: Option<String>,
    recipients: Option<Vec<Value>>,
    guild: Option<RawGuild>,
}

#[derive(Deserialize)]
struct RawMessage {
    #[serde(rename = "ID", alias = "id")]
    id: Option<Value>,
    #[serde(rename = "Timestamp", alias = "timestamp")]
    timestamp: Option<String>,
    #[serde(rename = "Contents", alias = "content")]
    contents: Option<String>,
}

fn value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub(crate) fn format_timestamp(ts: &str) -> String {
    if ts.is_empty() {
        return String::new();
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return dt.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string();
    }
    for fmt in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(ts, fmt) {
            let utc = Utc.from_utc_datetime(&naive);
            return utc.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }
    ts.to_string()
}

pub(crate) fn make_preview(content: &str) -> String {
    let mut out = String::with_capacity(content.len().min(96));
    for ch in content.chars().take(80) {
        out.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    out
}

pub fn parse_pack(folder_path: &Path) -> Result<PackData, String> {
    let messages_root = ["Messages", "messages"]
        .into_iter()
        .map(|name| folder_path.join(name))
        .find(|p| p.is_dir())
        .ok_or_else(|| {
            "\"Messages\" folder not found in pack. Make sure you selected the extracted (not ZIP) Discord pack folder.".to_string()
        })?;

    let mut channel_dirs: Vec<String> = fs::read_dir(&messages_root)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let valid_name = name
                .strip_prefix('c')
                .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()));
            if valid_name && entry.path().is_dir() {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    channel_dirs.sort_unstable();

    let mut servers: Vec<ServerGroup> = Vec::new();
    let mut server_indices: HashMap<String, usize> = HashMap::new();
    let mut dms: Vec<DmGroup> = Vec::new();
    let mut total_messages = 0usize;

    for channel_dir in channel_dirs {
        let channel_path = messages_root.join(&channel_dir);
        let messages_path = channel_path.join("messages.json");
        if !messages_path.is_file() {
            continue;
        }

        let channel_info_path = channel_path.join("channel.json");
        let channel_info = fs::read_to_string(&channel_info_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<RawChannelInfo>(&raw).ok());

        let Ok(raw_text) = fs::read_to_string(&messages_path) else {
            continue;
        };
        let Ok(raw_messages) = serde_json::from_str::<Vec<RawMessage>>(&raw_text) else {
            continue;
        };

        let type_str = channel_info
            .as_ref()
            .and_then(|c| c.kind.as_ref())
            .and_then(value_to_string)
            .unwrap_or_else(|| "GUILD_TEXT".to_string())
            .to_uppercase();
        let is_dm = matches!(type_str.as_str(), "DM" | "1" | "GROUP_DM" | "3");

        let channel_id = channel_info
            .as_ref()
            .and_then(|c| c.id.as_ref())
            .and_then(value_to_string)
            .unwrap_or_else(|| channel_dir[1..].to_string());

        let recipient_ids: Vec<String> = channel_info
            .as_ref()
            .and_then(|c| c.recipients.as_ref())
            .map(|list| list.iter().filter_map(value_to_string).collect())
            .unwrap_or_default();

        let guild_id = channel_info
            .as_ref()
            .and_then(|c| c.guild.as_ref())
            .and_then(|g| g.id.as_ref())
            .and_then(value_to_string);

        let guild_name = channel_info
            .as_ref()
            .and_then(|c| c.guild.as_ref())
            .and_then(|g| g.name.clone())
            .filter(|n| !n.is_empty());

        let channel_name = channel_info
            .as_ref()
            .and_then(|c| c.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| channel_dir.clone());

        let mut messages = Vec::with_capacity(raw_messages.len());
        for raw in raw_messages {
            let Some(id) = raw.id.as_ref().and_then(value_to_string) else {
                continue;
            };
            let timestamp = raw.timestamp.unwrap_or_default();
            let formatted_time = format_timestamp(&timestamp);
            let content = raw.contents.unwrap_or_default();
            let preview = make_preview(&content);
            messages.push(Message {
                id,
                formatted_time,
                content,
                preview,
                channel_id: channel_id.clone(),
                is_dm,
                recipient_ids: recipient_ids.clone(),
                channel_name: channel_name.clone(),
            });
        }

        total_messages += messages.len();

        if is_dm {
            let display_name = channel_info
                .as_ref()
                .and_then(|c| c.name.clone())
                .filter(|n| !n.is_empty())
                .or_else(|| recipient_ids.first().cloned())
                .unwrap_or(channel_dir);
            dms.push(DmGroup {
                name: display_name,
                messages,
            });
        } else {
            let resolved_guild_name = guild_name.unwrap_or_else(|| "Unknown server".to_string());
            let resolved_guild_id = guild_id.unwrap_or_else(|| "__unknown__".to_string());
            let idx = *server_indices
                .entry(resolved_guild_id)
                .or_insert_with(|| {
                    let next = servers.len();
                    servers.push(ServerGroup {
                        name: resolved_guild_name,
                        channels: Vec::new(),
                    });
                    next
                });
            servers[idx].channels.push(ChannelGroup { messages });
        }
    }

    Ok(PackData {
        servers,
        dms,
        total_messages,
    })
}

pub fn filter_pack(
    pack: &PackData,
    include_servers: bool,
    include_dms: bool,
    search: &str,
) -> Vec<FilteredMessage> {
    let needle = search.trim().to_lowercase();
    let mut out = Vec::new();

    if include_servers {
        for srv in &pack.servers {
            for ch in &srv.channels {
                for msg in &ch.messages {
                    if needle.is_empty() || msg.content.to_lowercase().contains(&needle) {
                        out.push(FilteredMessage {
                            msg: msg.clone(),
                            group_name: srv.name.clone(),
                            group_type: GroupType::Server,
                        });
                    }
                }
            }
        }
    }

    if include_dms {
        for dm in &pack.dms {
            for msg in &dm.messages {
                if needle.is_empty() || msg.content.to_lowercase().contains(&needle) {
                    out.push(FilteredMessage {
                        msg: msg.clone(),
                        group_name: dm.name.clone(),
                        group_type: GroupType::Dm,
                    });
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_pack_and_preserves_64bit_snowflakes() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("disvoke_test_{uniq}"));
        let srv_dir = root.join("Messages").join("c123456789012345678");
        let dm_dir = root.join("Messages").join("c987654321098765432");
        fs::create_dir_all(&srv_dir).unwrap();
        fs::create_dir_all(&dm_dir).unwrap();

        fs::write(
            srv_dir.join("channel.json"),
            r#"{"id":"123456789012345678","type":"GUILD_TEXT","name":"general","guild":{"id":"555","name":"Test Guild"}}"#,
        )
        .unwrap();
        fs::write(
            srv_dir.join("messages.json"),
            r#"[{"ID": 1234567890123456789, "Timestamp": "2025-01-10 12:00:00", "Contents": "hello world", "Attachments": ""}]"#,
        )
        .unwrap();

        fs::write(
            dm_dir.join("channel.json"),
            r#"{"id":"987654321098765432","type":1,"recipients":["111222333"]}"#,
        )
        .unwrap();
        fs::write(
            dm_dir.join("messages.json"),
            r#"[{"ID": "9876543210987654321", "Timestamp": "2025-01-11 13:30:00", "Contents": "secret dm", "Attachments": ""}]"#,
        )
        .unwrap();

        let pack = parse_pack(&root).unwrap();
        assert_eq!(pack.total_messages, 2);
        assert_eq!(pack.servers.len(), 1);
        assert_eq!(pack.servers[0].name, "Test Guild");
        assert_eq!(pack.servers[0].channels[0].messages[0].id, "1234567890123456789");
        assert_eq!(pack.dms.len(), 1);
        assert_eq!(pack.dms[0].name, "111222333");
        assert_eq!(pack.dms[0].messages[0].id, "9876543210987654321");

        let filtered = filter_pack(&pack, true, false, "HELLO");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].msg.id, "1234567890123456789");

        let _ = fs::remove_dir_all(&root);
    }
}
