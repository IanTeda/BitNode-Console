//! Journald Library Module
//!
//! This library provides journald interface functionality.

mod error;

/// Re-export Journald error type.
pub use error::Error;

/// Result type alias used across the auth module.
pub type Result<T> = std::result::Result<T, Error>;
