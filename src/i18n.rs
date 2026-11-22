use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const EN_JSON: &str = include_str!("../locales/en.json");
const PL_JSON: &str = include_str!("../locales/pl.json");

#[derive(Debug, Clone)]
pub struct LocaleInfo {
    pub code: String,
    pub name: String,
}

pub struct I18n {
    active: Value,
    fallback: Value,
    locales: BTreeMap<String, (String, Value)>,
}

impl I18n {
    pub fn new(initial_lang: &str) -> Self {
        let mut locales = BTreeMap::new();
        let en_val: Value = serde_json::from_str(EN_JSON).unwrap_or(Value::Null);
        let pl_val: Value = serde_json::from_str(PL_JSON).unwrap_or(Value::Null);

        locales.insert("en".to_string(), (locale_display_name("en", &en_val), en_val.clone()));
        locales.insert("pl".to_string(), (locale_display_name("pl", &pl_val), pl_val));

        for dir in external_locale_dirs() {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(code) = path.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
                    continue;
                };
                if let Ok(raw) = fs::read_to_string(&path)
                    && let Ok(val) = serde_json::from_str::<Value>(&raw)
                {
                    let name = locale_display_name(&code, &val);
                    locales.insert(code, (name, val));
                }
            }
        }

        let mut i18n = Self {
            active: en_val.clone(),
            fallback: en_val,
            locales,
        };
        i18n.set_lang(initial_lang);
        i18n
    }

    pub fn set_lang(&mut self, code: &str) {
        if let Some((_, val)) = self.locales.get(code) {
            self.active = val.clone();
        } else {
            self.active = self.fallback.clone();
        }
    }

    pub fn list_locales(&self) -> Vec<LocaleInfo> {
        self.locales
            .iter()
            .map(|(code, (name, _))| LocaleInfo {
                code: code.clone(),
                name: name.clone(),
            })
            .collect()
    }

    pub fn display_name<'a>(&'a self, code: &'a str) -> &'a str {
        self.locales
            .get(code)
            .map(|(name, _)| name.as_str())
            .unwrap_or(code)
    }

    pub fn t<'a>(&'a self, key: &str) -> &'a str {
        lookup(&self.active, key)
            .or_else(|| lookup(&self.fallback, key))
            .unwrap_or("")
    }

    pub fn fmt(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut out = self.t(key).to_string();
        for (k, v) in params {
            out = out.replace(&format!("{{{k}}}"), v);
        }
        out
    }
}

fn lookup<'a>(root: &'a Value, key: &str) -> Option<&'a str> {
    let mut cur = root;
    for part in key.split('.') {
        cur = cur.get(part)?;
    }
    cur.as_str()
}

fn locale_display_name(code: &str, val: &Value) -> String {
    if let Some(name) = val
        .get("_meta")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        && !name.is_empty()
    {
        return name.to_string();
    }
    match code {
        "en" => "English".to_string(),
        "pl" => "Polski".to_string(),
        other => other.to_string(),
    }
}

fn external_locale_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        dirs.push(parent.join("locales"));
    }
    dirs.push(PathBuf::from("locales"));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_locales_and_formats_placeholders() {
        let mut i18n = I18n::new("pl");
        assert_eq!(i18n.t("nav.dashboard"), "Pulpit");
        assert_eq!(
            i18n.fmt("progress.deleting", &[("current", "3"), ("total", "10")]),
            "Usuwanie 3 / 10..."
        );

        i18n.set_lang("unknown");
        assert_eq!(i18n.t("nav.dashboard"), "Dashboard");
    }
}
