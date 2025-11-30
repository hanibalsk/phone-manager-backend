use anyhow::Result;
use std::time::Duration;
use tracing::info;

mod app;
mod config;
mod error;
mod extractors;
mod jobs;
mod middleware;
mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = config::Config::load()?;

    // Initialize logging
    middleware::logging::init_logging(&config.logging);

    // Initialize Prometheus metrics
    middleware::metrics::init_metrics();
    info!("Prometheus metrics initialized");

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

    // Start job scheduler
    let mut scheduler = jobs::JobScheduler::new();
    scheduler.register(jobs::CleanupLocationsJob::new(
        pool.clone(),
        config.limits.location_retention_days,
    ));
    scheduler.register(jobs::RefreshViewsJob::new(pool.clone()));
    scheduler.register(jobs::PoolMetricsJob::new(pool.clone()));
    scheduler.start();

    // Build application
    let app = app::create_app(config.clone(), pool);

    // Start server
    let addr = config.socket_addr();
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Handle shutdown gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received shutdown signal");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Shutdown job scheduler
    scheduler.shutdown();
    scheduler.wait_for_shutdown(Duration::from_secs(30)).await;

    info!("Server shutdown complete");
    Ok(())
}
