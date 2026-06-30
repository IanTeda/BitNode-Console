//! Journald Library Crate Error
//!
//! This module provides error types for the journald library crate.

/// Errors that can occur in the journald library.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Catch-all for errors that do not fit a more specific variant.
    #[error("Journald  error: {0}")]
    Generic(String),
}
