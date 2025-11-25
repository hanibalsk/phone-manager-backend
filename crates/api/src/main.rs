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
    let db_config = persistence::db::DatabaseConfig {
        url: config.database.url.clone(),
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        connect_timeout_secs: config.database.connect_timeout_secs,
        idle_timeout_secs: config.database.idle_timeout_secs,
    };
    let pool = persistence::db::create_pool(&db_config).await?;

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
