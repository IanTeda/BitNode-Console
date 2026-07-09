//! Journals Library Module
//!
//! This library provides journals interface functionality.

// --- Module imports

mod connection;
mod domains;
mod error;
mod follow;
mod seek;

// --- Re-exports to flatten module hierarchy to the crate top level

/// Re-export Journals error type.
pub use error::Error;

/// Re-export journal domain types.
pub use domains::{Entry, FollowTail, Page, Priority, Query};

/// Re-export connection types.
pub use connection::Connection;

// -- Result type alias

/// Result type alias used across the journals module.
pub type Result<T> = std::result::Result<T, Error>;
