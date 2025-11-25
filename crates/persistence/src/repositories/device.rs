//! Device repository for database operations.

use sqlx::PgPool;

/// Repository for device-related database operations.
#[derive(Clone)]
pub struct DeviceRepository {
    pool: PgPool,
}

impl DeviceRepository {
    /// Creates a new DeviceRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Repository methods will be implemented in Story 2.1 (Device Registration API)
