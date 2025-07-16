pub mod logging;

use anyhow::Result;
use std::sync::Arc;

use crate::config::AppConfig;

pub struct ObservabilityManager {
    config: Arc<AppConfig>,
}

impl ObservabilityManager {
    pub async fn new(config: Arc<AppConfig>) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn get_config(&self) -> &Arc<AppConfig> {
        &self.config
    }
}
