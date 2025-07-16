use std::sync::Arc;

use anyhow::{Ok, Result};

mod config;
mod utils;
mod services;
mod models;
mod handlers;
mod dto;
mod middleware;

use axum::{Router, routing::{get, post, put}, middleware as axum_middleware};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use config::AppConfig;
use utils::ObservabilityManager;
use services::database::Database;
use tokio::signal;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::admin_head::{self, ApiDoc};
use crate::handlers::auth;
use crate::middleware::AuthMiddleware;

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
    /// Database service
    pub db: Database,
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

    let database = Database::new(
        &config.database.url,
        &config.database.name 
    ).await?;

    let state = AppState {
        db: database,
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
    // Protected API routes - require authentication and admin role
    let protected_api_routes = Router::new()
        .route("/api/assets", post(admin_head::create_asset))
        .route("/api/assets", get(admin_head::list_asset))
        .route("/api/assets/filter", get(admin_head::get_filtered_assets))
        .route("/api/assets/:id", put(admin_head::update_asset))
        .route("/api/assets/:id", axum::routing::delete(admin_head::delete_asset))
        .route("/api/upload", post(admin_head::upload_file))
        .layer(axum_middleware::from_fn(AuthMiddleware::require_admin_role))
        .layer(axum_middleware::from_fn_with_state(state.clone(), AuthMiddleware::auth_middleware));

    // Public auth routes - no authentication required
    let auth_routes = Router::new()
        .route("/", get(|| async { axum::response::Redirect::permanent("/login") }))
        .route("/login", get(auth::login_page))
        .route("/login", post(auth::login_handler));

    // Asset serving routes for file access
    let asset_routes = Router::new()
        .nest_service("/asset", ServeDir::new("asset"))
        .nest_service("/admin_head/asset", ServeDir::new("asset"));

    // Protected admin routes - require authentication and admin role
    let protected_admin_routes = Router::new()
        .route("/admin_head", get(admin_head::dashboard_page))
        .route("/admin_head/assets", get(admin_head::asset_management_page))
        .layer(axum_middleware::from_fn(AuthMiddleware::require_admin_role))
        .layer(axum_middleware::from_fn_with_state(state.clone(), AuthMiddleware::auth_middleware));

    // Protected director routes - require authentication and director role
    let protected_director_routes = Router::new()
        .route("/director", get(|| async { 
            axum::response::Html(include_str!("templates/director/base.html"))
        }))
        .layer(axum_middleware::from_fn(AuthMiddleware::require_director_role))
        .layer(axum_middleware::from_fn_with_state(state.clone(), AuthMiddleware::auth_middleware));

    // Static file serving - no authentication required
    let static_routes = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route_service("/project_list.yaml", ServeFile::new("project_list.yaml"));

    // API Documentation - Swagger UI
    let api_docs_routes = Router::new()
        .merge(SwaggerUi::new("/admin_head/api-docs")
            .url("/admin_head/api-docs/openapi.json", ApiDoc::openapi()));
            
    // API docs route for protected access
    let protected_api_docs_routes = Router::new()
        .merge(api_docs_routes)
        .layer(axum_middleware::from_fn(AuthMiddleware::require_admin_role))
        .layer(axum_middleware::from_fn_with_state(state.clone(), AuthMiddleware::auth_middleware));

    Router::new()
        .merge(protected_api_routes)
        .merge(auth_routes)
        .merge(asset_routes)
        .merge(protected_admin_routes)
        .merge(protected_director_routes)
        .merge(static_routes)
        .merge(protected_api_docs_routes)
        .with_state(state)
}