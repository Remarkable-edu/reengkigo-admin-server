use anyhow::Result;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Application configuration structure
///
/// Features:
/// - AppConfig
/// - DatabaseConfig

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub server: ServerConfig,
    pub external_api: ExternalApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub version: String,
    pub debug: bool,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// External API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiConfig {
    pub base_url: String,
    pub bucket: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                name: "reengkigo".to_string(),
                version: "1.0.0".to_string(),
                debug: true,
            },
            server: ServerConfig { 
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            external_api: ExternalApiConfig {
                base_url: "https://r2-api.reengki.com".to_string(),
                bucket: "reengki-archive".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        info!("Loading application configuration...");

        let config: AppConfig = Figment::new()
            // Start with default values
            .merge(Serialized::defaults(Self::default())) // Serialize된 AppConfig를 Provider로 감쌈
            // Override with config file if present
            .merge(Yaml::file("config.yaml"))
            // Override with environment variables
            .merge(Env::prefixed("APP_").split("_"))
            .extract()?;

        info!("Configuration loaded successfully");
        info!("name: {:?}", config.app.name);
        info!("External API: {}", config.external_api.base_url);
        info!("Bucket: {}", config.external_api.bucket);

        Ok(config)
    }
}
