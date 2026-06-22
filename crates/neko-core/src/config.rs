use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Local,
    Server,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AppConfig {
    pub mode: Option<Mode>,
    pub local: Option<LocalConfig>,
    pub client_server: Option<ClientServerConfig>,
    pub server: Option<ServerConfig>,
    pub llm: Option<LlmConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalConfig {
    pub db_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientServerConfig {
    pub api_base_url: String,
    #[serde(default)]
    pub auth_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub bind: String,
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default)]
    pub auth_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
        }
    }
}

impl Default for ClientServerConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8002/api/v1".to_string(),
            auth_token: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8002".to_string(),
            db_path: default_db_path(),
            auth_token: None,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-5.5".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }
        let text = toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn local_db_url(&self) -> Result<String> {
        let local = self.local.as_ref().context("missing [local] config")?;
        sqlite_url_from_path(&local.db_path)
    }

    pub fn server_db_url(&self) -> Result<String> {
        let server = self.server.as_ref().context("missing [server] config")?;
        sqlite_url_from_path(&server.db_path)
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn config_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".neko-words"))
}

pub fn default_sqlite_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("neko-words.sqlite3"))
}

pub fn expand_home(value: &str) -> Result<PathBuf> {
    if value == "~" {
        return home_dir();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }
    Ok(PathBuf::from(value))
}

fn sqlite_url_from_path(value: &str) -> Result<String> {
    let path = expand_home(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create data directory {}", parent.display()))?;
    }
    Ok(format!("sqlite://{}", path.display()))
}

fn default_db_path() -> String {
    "~/.neko-words/neko-words.sqlite3".to_string()
}

fn home_dir() -> Result<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .context("USERPROFILE is not set")
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")
    }
}
