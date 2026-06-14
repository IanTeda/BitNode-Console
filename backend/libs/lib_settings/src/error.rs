//! Settings Crate Error
//!
//! This module provides error types for the settings module.

use thiserror::Error;

/// Result type alias used across the settings module.
pub type SettingsResult<T> = std::result::Result<T, SettingsError>;

#[derive(Error, Debug)]
pub enum SettingsError {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = SettingsError::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }
}
