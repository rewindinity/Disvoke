use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BatchModeSettings {
    pub enabled: bool,
    pub min_batch: u32,
    pub max_batch: u32,
    pub min_break_min: u32,
    pub max_break_min: u32,
}

impl Default for BatchModeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            min_batch: 400,
            max_batch: 600,
            min_break_min: 10,
            max_break_min: 120,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EditContentMode {
    #[default]
    Custom,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EditBeforeDeleteSettings {
    pub enabled: bool,
    pub mode: EditContentMode,
    pub custom_text: String,
    pub random_length: u32,
}

impl Default for EditBeforeDeleteSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: EditContentMode::Custom,
            custom_text: "[deleted]".to_string(),
            random_length: 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub token: String,
    pub delay: u32,
    pub lang: String,
    pub history_path: String,
    pub batch_mode: BatchModeSettings,
    pub edit_before_delete: EditBeforeDeleteSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            token: String::new(),
            delay: 4,
            lang: "en".to_string(),
            history_path: String::new(),
            batch_mode: BatchModeSettings::default(),
            edit_before_delete: EditBeforeDeleteSettings::default(),
        }
    }
}

pub struct Store {
    file_path: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let file_path = ProjectDirs::from("com", "disvoke", "Disvoke")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .unwrap_or_else(|| PathBuf::from("settings.json"));
        Self { file_path }
    }

    pub fn load(&self) -> Settings {
        fs::read_to_string(&self.file_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, settings: &Settings) -> bool {
        if let Some(parent) = self.file_path.parent()
            && fs::create_dir_all(parent).is_err()
        {
            return false;
        }
        serde_json::to_string_pretty(settings)
            .ok()
            .and_then(|json| fs::write(&self.file_path, json).ok())
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_json_matches_electron_schema() {
        let json = r#"{
            "token": "abc.def",
            "delay": 7,
            "lang": "pl",
            "historyPath": "/tmp/h.json",
            "batchMode": {
                "enabled": true,
                "minBatch": 300,
                "maxBatch": 500,
                "minBreakMin": 15,
                "maxBreakMin": 90
            }
        }"#;
        let parsed: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.token, "abc.def");
        assert_eq!(parsed.delay, 7);
        assert_eq!(parsed.lang, "pl");
        assert_eq!(parsed.history_path, "/tmp/h.json");
        assert!(parsed.batch_mode.enabled);
        assert_eq!(parsed.batch_mode.min_batch, 300);
        assert_eq!(parsed.batch_mode.max_break_min, 90);
        assert!(!parsed.edit_before_delete.enabled);
        assert_eq!(parsed.edit_before_delete.mode, EditContentMode::Custom);
        assert_eq!(parsed.edit_before_delete.random_length, 16);
    }

    #[test]
    fn edit_before_delete_settings_roundtrip() {
        let mut settings = Settings::default();
        settings.edit_before_delete.enabled = true;
        settings.edit_before_delete.mode = EditContentMode::Random;
        settings.edit_before_delete.custom_text = "wiped".to_string();
        settings.edit_before_delete.random_length = 32;

        let json = serde_json::to_string(&settings).unwrap();
        let decoded: Settings = serde_json::from_str(&json).unwrap();
        assert!(decoded.edit_before_delete.enabled);
        assert_eq!(decoded.edit_before_delete.mode, EditContentMode::Random);
        assert_eq!(decoded.edit_before_delete.custom_text, "wiped");
        assert_eq!(decoded.edit_before_delete.random_length, 32);
    }
}
