//! Database connection pool management.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

/// Creates a PostgreSQL connection pool with the given configuration.
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect(&config.url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            url: "postgres://user:pass@localhost:5432/testdb".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }

    #[test]
    fn test_database_config_creation() {
        let config = create_test_config();
        assert_eq!(config.url, "postgres://user:pass@localhost:5432/testdb");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.idle_timeout_secs, 600);
    }

    #[test]
    fn test_database_config_clone() {
        let config = create_test_config();
        let cloned = config.clone();
        assert_eq!(cloned.url, config.url);
        assert_eq!(cloned.max_connections, config.max_connections);
        assert_eq!(cloned.min_connections, config.min_connections);
        assert_eq!(cloned.connect_timeout_secs, config.connect_timeout_secs);
        assert_eq!(cloned.idle_timeout_secs, config.idle_timeout_secs);
    }

    #[test]
    fn test_database_config_debug() {
        let config = create_test_config();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DatabaseConfig"));
        assert!(debug_str.contains("max_connections: 10"));
        assert!(debug_str.contains("min_connections: 2"));
    }

    #[test]
    fn test_database_config_default_values() {
        let config = DatabaseConfig {
            url: "postgres://localhost/db".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout_secs: 10,
            idle_timeout_secs: 300,
        };
        assert_eq!(config.max_connections, 5);
        assert!(config.max_connections >= config.min_connections);
    }

    #[test]
    fn test_database_config_production_values() {
        let config = DatabaseConfig {
            url: "postgres://prod:secret@db.example.com:5432/production".to_string(),
            max_connections: 100,
            min_connections: 10,
            connect_timeout_secs: 60,
            idle_timeout_secs: 1800,
        };
        assert_eq!(config.max_connections, 100);
        assert!(config.url.contains("production"));
    }

    #[test]
    fn test_database_config_url_formats() {
        // Test various URL formats are accepted
        let configs = vec![
            "postgres://localhost/db",
            "postgres://user@localhost/db",
            "postgres://user:pass@localhost/db",
            "postgres://user:pass@localhost:5432/db",
            "postgres://user:pass@localhost:5432/db?sslmode=require",
        ];

        for url in configs {
            let config = DatabaseConfig {
                url: url.to_string(),
                max_connections: 5,
                min_connections: 1,
                connect_timeout_secs: 10,
                idle_timeout_secs: 300,
            };
            assert!(!config.url.is_empty());
        }
    }

    #[test]
    fn test_database_config_connection_bounds() {
        let config = DatabaseConfig {
            url: "postgres://localhost/db".to_string(),
            max_connections: 50,
            min_connections: 5,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        };
        assert!(config.max_connections >= config.min_connections);
        assert!(config.connect_timeout_secs > 0);
        assert!(config.idle_timeout_secs > 0);
    }
}
