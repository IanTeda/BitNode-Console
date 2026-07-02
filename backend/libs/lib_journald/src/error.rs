//! Journald Library Crate Error
//!
//! This module provides error types for the journald library crate.

/// Errors that can occur in the journald library.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A systemd journal I/O error.
    #[error("Journald I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An integer conversion failed (e.g. journal timestamp or limit overflowed the target type).
    #[error("Journald integer conversion error: {0}")]
    IntConversion(#[from] std::num::TryFromIntError),

    /// Catch-all for errors that do not fit a more specific variant.
    #[error("Journald error: {0}")]
    Generic(String),
}
