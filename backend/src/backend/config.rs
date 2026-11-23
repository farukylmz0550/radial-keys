use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::presets::{self, BuiltinMacroConfig};
use super::shortcuts::{ActivationMode, Shortcut};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RadialConfig {
    pub version: u32,
    pub activation: ActivationConfig,
    pub slots: Vec<SlotConfig>,
    #[serde(default)]
    pub macro_settings: MacroSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct MacroSettings {
    pub single_instance: bool,
    pub timeout_seconds: Option<u64>,
}

impl Default for MacroSettings {
    fn default() -> Self {
        Self {
            single_instance: true,
            timeout_seconds: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActivationConfig {
    #[serde(rename = "type")]
    pub input_type: ActivationType,
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
    #[serde(default)]
    pub mode: ActivationMode,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mouse_button: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActivationType {
    #[default]
    Keyboard,
    Mouse,
}

fn default_key() -> String {
    "Y".into()
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            input_type: ActivationType::Mouse,
            key: default_key(),
            modifiers: vec!["Ctrl".into(), "Shift".into()],
            mode: ActivationMode::Toggle,
            enabled: false,
            mouse_button: Some("back".into()),
        }
    }
}

impl ActivationConfig {
    pub fn shortcut(&self) -> String {
        self.modifiers
            .iter()
            .cloned()
            .chain(std::iter::once(self.key.clone()))
            .collect::<Vec<_>>()
            .join("+")
    }

    pub fn set_shortcut(&mut self, value: &str) -> Result<(), ConfigError> {
        let parsed =
            Shortcut::parse(value).map_err(|error| ConfigError::Invalid(error.to_string()))?;
        let canonical = parsed.canonical();
        let mut parts = canonical.split('+').map(str::to_owned).collect::<Vec<_>>();
        self.key = parts.pop().expect("a parsed shortcut always has a key");
        self.modifiers = parts;
        self.input_type = ActivationType::Keyboard;
        self.mouse_button = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlotConfig {
    pub id: String,
    pub label: String,
    pub action: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub preset: Option<BuiltinMacroConfig>,
    #[serde(default)]
    pub script_sha256: Option<String>,
}

impl Default for RadialConfig {
    fn default() -> Self {
        Self {
            version: 1,
            activation: ActivationConfig::default(),
            slots: [
                "north",
                "north-east",
                "east",
                "south-east",
                "south",
                "south-west",
                "west",
                "north-west",
            ]
            .into_iter()
            .enumerate()
            .map(|(index, id)| SlotConfig {
                id: id.into(),
                label: match index {
                    0 => "Play / pause".into(),
                    1 => "Volume down".into(),
                    2 => "Volume up".into(),
                    3 => "Toggle mute".into(),
                    4 => "Next track".into(),
                    5 => "Previous track".into(),
                    6 => "Stop media".into(),
                    7 => "Spotify".into(),
                    _ => format!("Slot {}", index + 1),
                },
                action: String::new(),
                icon: match index {
                    0 => "qrc:/icons/media-play-pause.svg".into(),
                    1 => "qrc:/icons/volume-down.svg".into(),
                    2 => "qrc:/icons/volume-up.svg".into(),
                    3 => "qrc:/icons/volume-toggle.svg".into(),
                    4 => "qrc:/icons/media-next.svg".into(),
                    5 => "qrc:/icons/media-previous.svg".into(),
                    6 => "qrc:/icons/media-stop.svg".into(),
                    7 => "qrc:/icons/open-website.svg".into(),
                    _ => String::new(),
                },
                preset: match index {
                    0 => Some(BuiltinMacroConfig {
                        id: "media_play_pause".into(),
                        parameters: vec![],
                    }),
                    1 => Some(BuiltinMacroConfig {
                        id: "volume_down".into(),
                        parameters: vec![],
                    }),
                    2 => Some(BuiltinMacroConfig {
                        id: "volume_up".into(),
                        parameters: vec![],
                    }),
                    3 => Some(BuiltinMacroConfig {
                        id: "volume_toggle".into(),
                        parameters: vec![],
                    }),
                    4 => Some(BuiltinMacroConfig {
                        id: "media_next".into(),
                        parameters: vec![],
                    }),
                    5 => Some(BuiltinMacroConfig {
                        id: "media_previous".into(),
                        parameters: vec![],
                    }),
                    6 => Some(BuiltinMacroConfig {
                        id: "media_stop".into(),
                        parameters: vec![],
                    }),
                    7 => Some(BuiltinMacroConfig {
                        id: "open_website".into(),
                        parameters: vec!["https://open.spotify.com".into()],
                    }),
                    _ => None,
                },
                script_sha256: None,
            })
            .collect(),
            macro_settings: MacroSettings::default(),
        }
    }
}

impl RadialConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(ConfigError::Invalid(
                "unsupported configuration version".into(),
            ));
        }
        match self.activation.input_type {
            ActivationType::Keyboard => {
                let parsed = Shortcut::parse(&self.activation.shortcut())
                    .map_err(|error| ConfigError::Invalid(error.to_string()))?;
                if parsed.canonical() != self.activation.shortcut() {
                    return Err(ConfigError::Invalid(
                        "activation shortcut is not canonical".into(),
                    ));
                }
            }
            ActivationType::Mouse
                if self
                    .activation
                    .mouse_button
                    .as_deref()
                    .is_none_or(|button| {
                        !matches!(button, "left" | "right" | "middle" | "back" | "forward")
                    }) =>
            {
                return Err(ConfigError::Invalid(
                    "mouse activation requires a supported mouse_button".into(),
                ));
            }
            ActivationType::Mouse => {}
        }
        if self.slots.len() != 8 {
            return Err(ConfigError::Invalid(
                "exactly eight slots are required".into(),
            ));
        }
        if self.macro_settings.timeout_seconds == Some(0) {
            return Err(ConfigError::Invalid(
                "macro timeout must be greater than zero or disabled".into(),
            ));
        }
        let mut ids = std::collections::HashSet::new();
        for slot in &self.slots {
            if slot.id.trim().is_empty() || !ids.insert(slot.id.as_str()) {
                return Err(ConfigError::Invalid(
                    "slot identifiers must be non-empty and unique".into(),
                ));
            }
            if slot.label.trim().is_empty() {
                return Err(ConfigError::Invalid("slot labels must not be empty".into()));
            }
            if slot.action.len() > 4096
                || (!slot.action.is_empty()
                    && (!Path::new(&slot.action).is_absolute()
                        || Path::new(&slot.action)
                            .extension()
                            .is_none_or(|extension| extension != "py")))
            {
                return Err(ConfigError::Invalid(
                    "Python macro path must be an absolute .py path under 4096 bytes".into(),
                ));
            }
            if !slot.action.is_empty() && slot.preset.is_some() {
                return Err(ConfigError::Invalid(
                    "a slot cannot contain both a Python script and a built-in macro".into(),
                ));
            }
            if let Some(preset) = &slot.preset {
                presets::validate(preset).map_err(ConfigError::Invalid)?;
            }
            if slot.script_sha256.as_ref().is_some_and(|hash| {
                hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            }) {
                return Err(ConfigError::Invalid(
                    "script SHA-256 must be 64 hexadecimal characters".into(),
                ));
            }
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
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> anyhow::Result<Self> {
        let config_dir = super::paths::config_dir()
            .ok_or_else(|| anyhow::anyhow!("could not determine XDG config directory"))?;
        Ok(Self::new(config_dir.join("config.toml")))
    }

    pub fn load_or_default(&self) -> Result<RadialConfig, ConfigError> {
        match fs::symlink_metadata(&self.path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RadialConfig::default());
            }
            Err(error) => return Err(error.into()),
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(ConfigError::Invalid(
                    "configuration path must be a regular file, not a symlink or special file"
                        .into(),
                ));
            }
            Ok(metadata) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & 0o077 != 0 {
                        fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
                    }
                }
            }
        }
        let text = fs::read_to_string(&self.path)?;
        let mut config: RadialConfig = toml::from_str(&text)?;
        config.validate()?;
        let defaults = RadialConfig::default();
        let mut changed = false;
        for (index, slot) in config.slots.iter_mut().enumerate() {
            if slot.action.is_empty() && slot.preset.is_none() {
                let default_slot = &defaults.slots[index];
                slot.label = default_slot.label.clone();
                slot.icon = default_slot.icon.clone();
                slot.preset = default_slot.preset.clone();
                changed = true;
            }
        }
        for slot in &mut config.slots {
            if let Some(preset) = &mut slot.preset
                && preset.id == "open_website"
                && preset
                    .parameters
                    .first()
                    .is_some_and(|url| url == "https://example.com")
            {
                preset.parameters[0] = "https://www.youtube.com".into();
                slot.label = "YouTube".into();
                changed = true;
            }
            if !slot.action.is_empty()
                && slot.script_sha256.is_none()
                && let Ok(path) = super::macros::validate_script_path(&slot.action)
                && let Ok(hash) = super::macros::sha256_file(&path)
            {
                slot.action = path.display().to_string();
                slot.script_sha256 = Some(hash);
                changed = true;
            }
        }
        if changed {
            self.save(&config)?;
        }
        Ok(config)
    }

    pub fn save(&self, config: &RadialConfig) -> Result<(), ConfigError> {
        config.validate()?;
        let text = toml::to_string_pretty(config)?;
        let parent = self.path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
            if let Ok(metadata) = fs::symlink_metadata(&self.path)
                && (metadata.file_type().is_symlink() || !metadata.is_file())
            {
                return Err(ConfigError::Invalid(
                    "configuration path must be a regular file, not a symlink or special file"
                        .into(),
                ));
            }
        }
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            temporary
                .as_file()
                .set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        temporary.write_all(text.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_eight_slots_and_toggle_ctrl_shift_y() {
        let config = RadialConfig::default();
        assert_eq!(config.slots.len(), 8);
        assert!(config.slots[4..].iter().all(|slot| slot.icon.is_empty()));
        assert_eq!(
            config
                .slots
                .iter()
                .filter(|slot| slot.preset.is_some())
                .count(),
            4
        );
        assert_eq!(config.activation.shortcut(), "Ctrl+Shift+Y");
        assert_eq!(config.activation.input_type, ActivationType::Mouse);
        assert_eq!(config.activation.mouse_button.as_deref(), Some("back"));
        assert_eq!(config.activation.mode, ActivationMode::Toggle);
        config.validate().unwrap();
    }

    #[test]
    fn old_config_without_icon_fields_loads_with_empty_icons() {
        let mut config = RadialConfig::default();
        for slot in &mut config.slots {
            slot.icon.clear();
        }
        let current = toml::to_string(&config).unwrap();
        let legacy = current.replace("icon = \"\"\n", "");
        let parsed: RadialConfig = toml::from_str(&legacy).unwrap();
        assert!(parsed.slots.iter().all(|slot| slot.icon.is_empty()));
    }

    #[test]
    fn rejects_wrong_slot_count_and_invalid_shortcut() {
        let mut config = RadialConfig::default();
        config.slots.pop();
        assert!(config.validate().is_err());
        config = RadialConfig::default();
        assert!(config.activation.set_shortcut("Ctrl+Escape").is_err());
    }

    #[test]
    fn rejects_invalid_timeout_and_conflicting_macro_sources() {
        let mut config = RadialConfig::default();
        config.macro_settings.timeout_seconds = Some(0);
        assert!(config.validate().is_err());
        config = RadialConfig::default();
        config.slots[0].action = "/tmp/macro.py".into();
        config.slots[0].preset = Some(BuiltinMacroConfig {
            id: "media_next".into(),
            parameters: vec![],
        });
        assert!(config.validate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_config_files_without_following_them() {
        let directory = tempfile::tempdir().unwrap();
        let actual = directory.path().join("actual.toml");
        fs::write(&actual, "not a configuration").unwrap();
        let link = directory.path().join("config.toml");
        std::os::unix::fs::symlink(&actual, &link).unwrap();
        assert!(
            ConfigStore::new(link)
                .load_or_default()
                .unwrap_err()
                .to_string()
                .contains("regular file")
        );
        assert_eq!(fs::read_to_string(actual).unwrap(), "not a configuration");
    }

    #[cfg(unix)]
    #[test]
    fn restricts_existing_config_permissions_on_load() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(&path, toml::to_string(&RadialConfig::default()).unwrap()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        ConfigStore::new(&path).load_or_default().unwrap();
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn mouse_activation_requires_a_supported_button() {
        let mut config = RadialConfig::default();
        config.activation.input_type = ActivationType::Mouse;
        config.activation.mouse_button = Some("back".into());
        config.validate().unwrap();
        config.activation.mouse_button = Some("button-99".into());
        assert!(config.validate().is_err());
    }

    #[test]
    fn saves_atomically_and_recovers_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().join("nested/config.toml"));
        assert_eq!(store.load_or_default().unwrap(), RadialConfig::default());
        let mut config = RadialConfig::default();
        config.activation.set_shortcut("Ctrl+Space").unwrap();
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
