use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const EMBEDDED_LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.json")),
    ("pl", include_str!("../locales/pl.json")),
    ("de", include_str!("../locales/de.json")),
    ("nl", include_str!("../locales/nl.json")),
    ("cs", include_str!("../locales/cs.json")),
    ("sk", include_str!("../locales/sk.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("it", include_str!("../locales/it.json")),
    ("es", include_str!("../locales/es.json")),
    ("fr", include_str!("../locales/fr.json")),
];

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
        let mut fallback = Value::Null;

        for &(code, raw) in EMBEDDED_LOCALES {
            let val: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
            if code == "en" {
                fallback = val.clone();
            }
            locales.insert(code.to_string(), (locale_display_name(code, &val), val));
        }

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
            active: fallback.clone(),
            fallback,
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
        "cs" => "Čeština".to_string(),
        "de" => "Deutsch".to_string(),
        "en" => "English".to_string(),
        "es" => "Español".to_string(),
        "fr" => "Français".to_string(),
        "it" => "Italiano".to_string(),
        "nl" => "Nederlands".to_string(),
        "pl" => "Polski".to_string(),
        "ru" => "Русский".to_string(),
        "sk" => "Slovenčina".to_string(),
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
    use std::collections::BTreeSet;

    fn collect_keys(val: &Value, prefix: &str, out: &mut BTreeMap<String, String>) {
        match val {
            Value::Object(map) => {
                for (k, v) in map {
                    let path = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    collect_keys(v, &path, out);
                }
            }
            Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            }
            _ => {}
        }
    }

    fn extract_placeholders(s: &str) -> BTreeSet<String> {
        let mut set = BTreeSet::new();
        let mut rest = s;
        while let Some(start) = rest.find('{') {
            rest = &rest[start + 1..];
            if let Some(end) = rest.find('}') {
                set.insert(rest[..end].to_string());
                rest = &rest[end + 1..];
            } else {
                break;
            }
        }
        set
    }

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

    #[test]
    fn all_embedded_locales_are_valid_and_complete() {
        let en_val: Value = serde_json::from_str(EMBEDDED_LOCALES[0].1).unwrap();
        let mut en_keys = BTreeMap::new();
        collect_keys(&en_val, "", &mut en_keys);
        assert!(!en_keys.is_empty());

        for &(code, raw) in EMBEDDED_LOCALES {
            let val: Value = serde_json::from_str(raw)
                .unwrap_or_else(|e| panic!("invalid JSON in {code}.json: {e}"));
            let mut loc_keys = BTreeMap::new();
            collect_keys(&val, "", &mut loc_keys);

            for (key, en_str) in &en_keys {
                let loc_str = loc_keys
                    .get(key)
                    .unwrap_or_else(|| panic!("missing key '{key}' in {code}.json"));
                assert!(
                    !loc_str.trim().is_empty(),
                    "empty translation for '{key}' in {code}.json"
                );
                assert_eq!(
                    extract_placeholders(en_str),
                    extract_placeholders(loc_str),
                    "placeholder mismatch for '{key}' in {code}.json"
                );
            }
        }
    }
}
