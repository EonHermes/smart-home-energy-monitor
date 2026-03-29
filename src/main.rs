mod api;
mod config;
mod database;
mod models;
mod services;
mod utils;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "smart_home_energy_monitor=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let cfg = config::Config::load()?;
    
    tracing::info!("Starting Smart Home Energy Monitor v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Server listening on {}:{}", cfg.server.host, cfg.server.port);

    // Initialize database
    database::init(&cfg.database.path).await?;

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .merge(api::consumption::router())
        .merge(api::predictions::router())
        .merge(api::optimizations::router());

    // Run server
    let addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    
    tracing::info!("API endpoints available at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Smart Home Energy Monitor API ⚡\n\nEndpoints:\n  POST /api/consumption - Record energy usage\n  GET  /api/consumption - Get consumption history\n  GET  /api/predictions - Get ML predictions\n  GET  /api/optimizations - Get optimization suggestions"
}
