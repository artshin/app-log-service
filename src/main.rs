//! Log Server - Development logging service
//!
//! HTTP server for receiving, storing, and displaying log entries from Swift clients.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use colored::Colorize;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod auth;
mod buffer;
mod config;
mod display;
mod handlers;
// mod html; // Deprecated - replaced by Askama templates
mod models;
mod request_manager;
mod storage;
mod tags;

use auth::JwtValidator;
use buffer::LogBuffer;
use config::Config;
use request_manager::RequestManager;
use storage::LogStorage;

/// Application state shared across handlers
pub struct AppState {
    pub buffer: LogBuffer,
    pub verbose: bool,
    pub request_manager: RequestManager,
    pub storage: LogStorage,
    pub jwt_validator: Option<JwtValidator>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (for server's own logs)
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // Load configuration
    let config = Config::from_env();

    // Initialize JWT validator if public key path is provided
    let jwt_validator = config
        .jwt_public_key_path
        .as_ref()
        .and_then(|path| match JwtValidator::from_pem_file(path) {
            Ok(validator) => {
                info!("JWT authentication enabled");
                Some(validator)
            }
            Err(e) => {
                tracing::warn!("Failed to load JWT public key: {}. Protected endpoints will not work.", e);
                None
            }
        });

    // Initialize log storage
    let storage = LogStorage::new(config.upload_dir.clone()).map_err(|e| {
        format!("Failed to initialize log storage: {}", e)
    })?;

    // Create shared state
    let state = Arc::new(AppState {
        buffer: LogBuffer::new(config.capacity),
        verbose: config.verbose,
        request_manager: RequestManager::new(),
        storage,
        jwt_validator,
    });

    // Build router
    let app = Router::new()
        // Public endpoints
        .route("/", get(handlers::handle_root))
        .route("/info", get(handlers::handle_info))
        .route("/logs", post(handlers::handle_receive_log))
        .route("/logs", get(handlers::handle_get_all_logs))
        .route("/logs", delete(handlers::handle_clear_logs))
        .route("/stream", get(handlers::handle_stream))
        // Protected endpoints (require JWT)
        .route("/logs/request", post(handlers::handle_create_request))
        .route("/logs/poll", get(handlers::handle_poll))
        .route("/logs/upload", post(handlers::handle_upload))
        .route("/logs/uploads", get(handlers::handle_list_uploads))
        .route("/logs/uploads/:request_id", get(handlers::handle_get_upload))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    // Server address
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

    // Print startup banner
    println!();
    println!("{}", "Log Server (Rust) starting...".green());
    println!("Listening on {}", addr.to_string().cyan());
    println!("Buffer capacity: {} entries", config.capacity);
    println!("Upload directory: {}", config.upload_dir.display());
    if config.verbose {
        println!("Verbose mode: {}", "ON".green());
    } else {
        println!("Verbose mode: OFF (set VERBOSE=1 for metadata)");
    }
    if config.jwt_public_key_path.is_some() {
        println!("Authentication: {}", "ENABLED".green());
    } else {
        println!("Authentication: {} (protected endpoints disabled)", "DISABLED".yellow());
    }
    println!();

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!();
    println!("Shutting down server...");
    info!("Goodbye!");

    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
