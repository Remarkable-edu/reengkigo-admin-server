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
    pub database: DataBaseConfigs,
    pub server: ServerConfig,
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub version: String,
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBaseConfigs {
    pub url: String,
    pub name: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                name: "reengkigo".to_string(),
                version: "1.0,0".to_string(),
                debug: true,
            },
            database: DataBaseConfigs {
                url: "mongodb://mongo:27017".to_string(),
                name: "admin_system".to_string(),
            },
            server: ServerConfig { 
                host: "0.0.0.0".to_string(),
                port: 3000,
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
        info!("Database: {}", config.database.url);

        Ok(config)
    }
}
