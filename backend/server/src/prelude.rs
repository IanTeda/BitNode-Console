//! Prelude module for the server.

pub use crate:: error::ServerError;

// Alias for Result type with ServerError as the error type.
pub type ServerResult<T> = std::result::Result<T, ServerError>;
