//! Journald Library Module
//!
//! This library provides journald interface functionality.

// --- Module imports

mod connection;
mod domains;
mod error;
mod query;
mod seek;

// --- Re-exports to flatten module hierarchy to the crate top level

/// Re-export Journald error type.
pub use error::Error;

/// Re-export journal domain types.
pub use domains::{JournalEntry, JournalPage, JournalPriority};

/// Re-export connection types.
pub use connection::JournalConnection;

/// Re-export seek types.
pub use query::JournalQuery;

// -- Result type alias

/// Result type alias used across the journald module.
pub type Result<T> = std::result::Result<T, Error>;
