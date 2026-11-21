use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::{Path, PathBuf}};
use thiserror::Error;

use crate::shortcuts::{ActivationMode, Shortcut};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RadialConfig {
    pub version: u32,
    pub shortcut: String,
    pub mode: ActivationMode,
    pub slots: Vec<SlotConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlotConfig {
    pub id: String,
    pub label: String,
    pub action: String,
}

impl Default for RadialConfig {
    fn default() -> Self {
        Self {
            version: 1,
            shortcut: "X".into(),
            mode: ActivationMode::Hold,
            slots: ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"]
                .into_iter()
                .enumerate()
                .map(|(index, id)| SlotConfig {
                    id: id.into(),
                    label: format!("Slot {}", index + 1),
                    action: String::new(),
                })
                .collect(),
        }
    }
}

impl RadialConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 { return Err(ConfigError::Invalid("unsupported configuration version".into())); }
        Shortcut::parse(&self.shortcut).map_err(|e| ConfigError::Invalid(e.to_string()))?;
        if self.slots.len() != 8 { return Err(ConfigError::Invalid("exactly eight slots are required".into())); }
        let mut ids = std::collections::HashSet::new();
        for slot in &self.slots {
            if slot.id.trim().is_empty() || !ids.insert(slot.id.as_str()) {
                return Err(ConfigError::Invalid("slot identifiers must be non-empty and unique".into()));
            }
            if slot.label.trim().is_empty() { return Err(ConfigError::Invalid("slot labels must not be empty".into())); }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("configuration I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("configuration parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("configuration serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone)]
pub struct ConfigStore { path: PathBuf }

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }

    pub fn default_path() -> anyhow::Result<Self> {
        let dirs = directories::ProjectDirs::from("org", "radial-keys", "radial-keys")
            .ok_or_else(|| anyhow::anyhow!("could not determine XDG config directory"))?;
        Ok(Self::new(dirs.config_dir().join("config.toml")))
    }

    pub fn load_or_default(&self) -> Result<RadialConfig, ConfigError> {
        if !self.path.exists() { return Ok(RadialConfig::default()); }
        let text = fs::read_to_string(&self.path)?;
        let config: RadialConfig = toml::from_str(&text)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, config: &RadialConfig) -> Result<(), ConfigError> {
        config.validate()?;
        let text = toml::to_string_pretty(config)?;
        let parent = self.path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile_path(&self.path);
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        let result = (|| -> Result<(), std::io::Error> {
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            if let Ok(dir) = fs::File::open(parent) { let _ = dir.sync_all(); }
            Ok(())
        })();
        if result.is_err() { let _ = fs::remove_file(&temporary); }
        result?;
        temporary.clear();
        Ok(())
    }
}

fn tempfile_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{name}.{}.tmp", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_eight_slots_and_hold_x() {
        let config = RadialConfig::default();
        assert_eq!(config.slots.len(), 8);
        assert_eq!(config.shortcut, "X");
        assert_eq!(config.mode, ActivationMode::Hold);
        config.validate().unwrap();
    }

    #[test]
    fn rejects_wrong_slot_count_and_invalid_shortcut() {
        let mut config = RadialConfig::default();
        config.slots.pop();
        assert!(config.validate().is_err());
        config = RadialConfig::default();
        config.shortcut = "Ctrl+Escape".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn saves_atomically_and_recovers_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().join("nested/config.toml"));
        assert_eq!(store.load_or_default().unwrap(), RadialConfig::default());
        let mut config = RadialConfig::default();
        config.shortcut = "Ctrl+Space".into();
        store.save(&config).unwrap();
        assert_eq!(store.load_or_default().unwrap(), config);
    }

    #[test]
    fn corrupt_file_is_reported_without_overwriting_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "not = [valid").unwrap();
        assert!(ConfigStore::new(&path).load_or_default().is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "not = [valid");
    }
}
