//! Logging initialization and configuration.

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use crate::config::LoggingConfig;

/// Initializes the logging subsystem based on configuration.
pub fn init_logging(config: &LoggingConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match config.format.as_str() {
        "json" => {
            let json_layer = fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true)
                .with_target(true);
            subscriber.with(json_layer).init();
        }
        _ => {
            let pretty_layer = fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(true);
            subscriber.with(pretty_layer).init();
        }
    }
}
