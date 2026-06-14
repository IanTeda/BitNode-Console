//! Config Crate Error
//!
//! This module provides error types for the configuration module.

use thiserror::Error;

/// Result type alias used across configuration module.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[derive(Error, Debug)]
pub enum ConfigError {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = ConfigError::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }
}
