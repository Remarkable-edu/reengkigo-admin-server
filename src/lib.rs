pub mod config;
pub mod utils;
pub mod services;
pub mod models;
pub mod handlers;
pub mod dto;
pub mod middleware;


use std::sync::Arc;
pub use utils::ObservabilityManager;
pub use services::database::Database;
pub use config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<AppConfig>,
    pub observability: Arc<ObservabilityManager>,
}