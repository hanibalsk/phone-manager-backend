use anyhow::Result;
use tracing::info;

mod app;
mod config;
mod error;
mod extractors;
mod middleware;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = config::Config::load()?;

    // Initialize logging
    middleware::logging::init_logging(&config.logging);

    info!("Starting Phone Manager API v{}", env!("CARGO_PKG_VERSION"));

    // Create database pool
    let pool = persistence::db::create_pool(&config.database).await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("../persistence/src/migrations")
        .run(&pool)
        .await?;
    info!("Migrations completed");

    // Build application
    let app = app::create_app(config.clone(), pool);

    // Start server
    let addr = config.socket_addr();
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
