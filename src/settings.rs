use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub auto_sync_seconds: u64,
    pub all_days: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_sync_seconds: 1,
            all_days: false,
        }
    }
}

impl Settings {
    pub fn normalized(mut self) -> Self {
        self.auto_sync_seconds = match self.auto_sync_seconds {
            0 | 1 | 5 | 15 | 60 => self.auto_sync_seconds,
            _ => 1,
        };
        self
    }
}

pub const SYNC_OPTIONS: [(u64, &str); 5] = [
    (0, "Off"),
    (1, "1 second"),
    (5, "5 seconds"),
    (15, "15 seconds"),
    (60, "60 seconds"),
];

pub fn config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("OTD_CONFIG_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("config.json");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(dir) = std::env::var("APPDATA") {
            if !dir.is_empty() {
                return PathBuf::from(dir)
                    .join("opencode-token-dashboard")
                    .join("config.json");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("opencode-token-dashboard")
                    .join("config.json");
            }
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("opencode-token-dashboard")
        .join("config.json")
}

pub fn pricing_path() -> PathBuf {
    config_path()
        .parent()
        .map(|parent| parent.join("pricing.json"))
        .unwrap_or_else(|| PathBuf::from("pricing.json"))
}

pub fn load_pricing_from(path: &Path) -> crate::cost::Pricing {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<crate::cost::Pricing>(&content).ok())
        .unwrap_or_default()
}

pub fn load_from(path: &Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Settings>(&content).ok())
        .map(Settings::normalized)
        .unwrap_or_default()
}

pub fn save_to(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_normalization() {
        assert_eq!(Settings::default().auto_sync_seconds, 1);
        assert_eq!(
            Settings {
                auto_sync_seconds: 7,
                all_days: false
            }
            .normalized()
            .auto_sync_seconds,
            1
        );
        assert_eq!(
            Settings {
                auto_sync_seconds: 15,
                all_days: true
            }
            .normalized()
            .auto_sync_seconds,
            15
        );
    }

    #[test]
    fn saves_and_loads() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("otd-settings-{nanos}.json"));
        let settings = Settings {
            auto_sync_seconds: 15,
            all_days: true,
        };
        save_to(&path, &settings).unwrap();
        assert_eq!(load_from(&path), settings);
        let _ = std::fs::remove_file(path);
    }
}
