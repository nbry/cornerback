use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

fn default_send_to_target() -> bool {
    false
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub target_server: TargetServerConfig,
    pub event_store: EventStoreConfig,
    #[serde(default = "default_send_to_target")]
    pub send_to_target_by_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetServerConfig {
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventStoreConfig {
    Postgres { database_url: String },
    InMemory,
    Custom { name: String },
}

impl AppConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;

        let config = serde_json::from_str::<Self>(&raw)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;

        Ok(config)
    }
}
