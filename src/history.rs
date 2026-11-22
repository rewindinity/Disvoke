use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub const AUTO_SAVE_INTERVAL: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPayload {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub saved_at: String,
    pub deleted_ids: Vec<String>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Default)]
pub struct DeletionHistory {
    pub deleted_ids: HashSet<String>,
    pub history_path: String,
    pub auto_save_counter: usize,
}

impl DeletionHistory {
    pub fn save_to_disk(&self) -> bool {
        if self.history_path.trim().is_empty() {
            return false;
        }
        let mut ids: Vec<String> = self.deleted_ids.iter().cloned().collect();
        ids.sort_unstable();
        let payload = HistoryPayload {
            version: 1,
            saved_at: Utc::now().to_rfc3339(),
            deleted_ids: ids,
        };
        serde_json::to_string_pretty(&payload)
            .ok()
            .and_then(|json| fs::write(&self.history_path, json).ok())
            .is_some()
    }

    pub fn load_from_path(path: &Path) -> Option<Vec<String>> {
        let raw = fs::read_to_string(path).ok()?;
        let parsed: HistoryPayload = serde_json::from_str(&raw).ok()?;
        Some(parsed.deleted_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn roundtrip_history_file() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("disvoke_hist_{uniq}.json"));
        let mut hist = DeletionHistory {
            history_path: path.display().to_string(),
            ..Default::default()
        };
        hist.deleted_ids.insert("1234567890123456789".to_string());
        hist.deleted_ids.insert("9876543210987654321".to_string());

        assert!(hist.save_to_disk());
        let loaded = DeletionHistory::load_from_path(&path).unwrap();
        assert_eq!(
            loaded,
            vec![
                "1234567890123456789".to_string(),
                "9876543210987654321".to_string()
            ]
        );
        let _ = fs::remove_file(&path);
    }
}
