//! Journald Library Module
//!
//! This library provides journald interface functionality.

mod domains;
mod error;

/// Re-export Journald error type.
pub use error::Error;

/// Re-export journal domain types.
pub use domains::{JournalEntry, JournalPriority};

/// Result type alias used across the auth module.
pub type Result<T> = std::result::Result<T, Error>;
