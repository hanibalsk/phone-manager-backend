//! Persistence layer for Phone Manager backend.
//!
//! This crate contains:
//! - Database connection management
//! - Entity definitions (database row mappings)
//! - Repository implementations
//! - Database metrics collection

pub mod db;
pub mod entities;
pub mod metrics;
pub mod repositories;
