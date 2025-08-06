use std::sync::Arc;

use anyhow::{Ok, Result};

mod config;
mod utils;
mod services;
mod models;
mod handlers;
mod dto;
mod middleware;

use axum::{Router, routing::{get, post}};
use axum::middleware as axum_middleware;
use axum::extract::DefaultBodyLimit;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use config::AppConfig;
use utils::ObservabilityManager;
use services::file::FileService;
use tokio::signal;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::{auth, file, dashboard};
use crate::middleware::auth::AuthMiddleware;
use server_test::ApiDoc;

/// Graceful shutdown signal handler
/// 
/// Handles shutdown signals gracefully, allowing in-flight requests to complete
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully...");
        },
    }
}


#[derive(Clone)]
pub struct AppState {
    /// File service
    pub file_service: Arc<FileService>,
    /// Application configuration
    pub config: Arc<AppConfig>,
    /// Observability manager
    pub observability: Arc<ObservabilityManager>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    utils::logging::init_logging()?;
    
    let config = Arc::new(AppConfig::load()?);
    let observability = Arc::new(ObservabilityManager::new(config.clone()).await?);

    let file_service = Arc::new(FileService::new(
        config.external_api.base_url.clone(),
        config.external_api.bucket.clone()
    ));

    let state = AppState {
        file_service,
        config: config.clone(),
        observability: observability.clone(),
    };

    let app = create_router(state);

    let addr = SocketAddr::from((config.server.host.parse::<std::net::IpAddr>()?, config.server.port));
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let shutdown_signal = shutdown_signal();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}


fn create_router(state: AppState) -> Router {
    // File API routes - no authentication for now
    let file_api_routes = Router::new()
        .route("/upload", post(file::upload_file))
        .route("/all-file", get(file::list_files))
        .route("/delete-file", post(file::delete_file))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024)) // 2GB limit for file uploads
        .with_state(state.file_service.clone());

    // Public auth routes - no authentication required
    let auth_routes = Router::new()
        .route("/", get(auth::root_handler))
        .route("/login", get(auth::login_page))
        .route("/login", post(auth::login_handler));

    // Admin dashboard routes - authentication required
    let admin_dashboard_routes = Router::new()
        .route("/dashboard", get(dashboard::dashboard_main))
        .route("/dashboard/assets", get(dashboard::dashboard_asset))
        .route("/api/assets", post(dashboard::create_asset))
        .route("/api/folders", get(dashboard::get_root_folders))
        .route("/api/folders/*path", get(dashboard::get_folder_contents))
        .route("/api/delete-item", post(dashboard::delete_item))
        .route("/api/subtitle/:book_id/:title", get(dashboard::get_subtitle_data))
        .route("/api/image/:book_id/:title", get(dashboard::get_image_content))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024)) // 2GB limit for asset uploads
        .layer(axum_middleware::from_fn(AuthMiddleware::auth_middleware));

    // Static file serving - no authentication required
    let static_routes = Router::new()
        .route_service("/project_list.yaml", ServeFile::new("project_list.yaml"))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/asset", ServeDir::new("assets").fallback(ServeFile::new("assets/placeholder.png")));

    // API Documentation - Swagger UI
    let api_docs = SwaggerUi::new("/api-docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    Router::new()
        .merge(file_api_routes)
        .merge(auth_routes)
        .merge(admin_dashboard_routes)
        .merge(static_routes)
        .merge(api_docs)
        .with_state(state)
}