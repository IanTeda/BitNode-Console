//! Prelude module for the server.

pub use crate::error::Error;

// Alias for Result type with ServerError as the error type.
pub type Result<T> = std::result::Result<T, Error>;
