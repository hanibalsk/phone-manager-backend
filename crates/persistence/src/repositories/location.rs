//! Location repository for database operations.

use sqlx::PgPool;

/// Repository for location-related database operations.
#[derive(Clone)]
pub struct LocationRepository {
    pool: PgPool,
}

impl LocationRepository {
    /// Creates a new LocationRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Repository methods will be implemented in Story 3.1 (Single Location Upload API)
