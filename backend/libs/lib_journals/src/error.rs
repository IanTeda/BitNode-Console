//! Journals Library Crate Error
//!
//! This module provides error types for the journals library crate.

/// Errors that can occur in the journals library.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A systemd journal I/O error.
    #[error("Journals I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An integer conversion failed (e.g. journal timestamp or limit overflowed the target type).
    #[error("Journals integer conversion error: {0}")]
    IntConversion(#[from] std::num::TryFromIntError),

    /// Catch-all for errors that do not fit a more specific variant.
    #[error("Journals error: {0}")]
    Generic(String),
}
