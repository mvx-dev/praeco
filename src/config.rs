use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Validation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not config file: {e}"),
            ConfigError::Parse(e) => write!(f, "could not parse config file: {e}"),
            ConfigError::Validation(msg) => write!(f, "invalid config: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Parse(e) => Some(e),
            ConfigError::Validation(_) => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::Parse(value)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub batteries: HashMap<String, BatteryConfig>,
}

impl Config {
    fn validate(&self) -> Result<(), ConfigError> {
        for (id, battery) in &self.batteries {
            if battery.uevent_name.trim().is_empty() {
                return Err(ConfigError::Validation(format!(
                    "battery '{id}': uevent_name is empty"
                )));
            }

            for rule in &battery.notify {
                if let NotifyRule::Capacity { percent, .. } = rule {
                    if *percent > 100 {
                        // TODO make this log a warning and set to some value
                        return Err(ConfigError::Validation(format!(
                            "battery '{id}': capacity percent {percent} is out of range (0-100)"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    // Lifetime of a notification
    pub display_time: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig { display_time: 5000 }
    }
}

#[derive(Debug, Deserialize)]
pub struct BatteryConfig {
    // Name of the entry under /sys/class/power_supply, e.g. "BAT1"
    pub uevent_name: String,
    // List of notifications to be sent for this battery
    #[serde(default)]
    pub notify: Vec<NotifyRule>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifyRule {
    // Notifications based on estimated remaining duration
    Time {
        minutes: u32,
        #[serde(default)]
        level: Level,
        message: Option<String>,
    },
    Capacity {
        percent: u8,
        #[serde(default)]
        level: Level,
        message: Option<String>,
    },
}

impl NotifyRule {
    pub fn level(&self) -> Level {
        match self {
            NotifyRule::Time { level, .. } => *level,
            NotifyRule::Capacity { level, .. } => *level,
        }
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            NotifyRule::Time { message, .. } => message.as_deref(),
            NotifyRule::Capacity { message, .. } => message.as_deref(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Info,
    #[default]
    Warning,
    Critical,
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let raw = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&raw)?;
    config.validate()?;
    Ok(config)
}
