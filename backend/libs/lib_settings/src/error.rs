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

    // Error encountered while parsing setting sources.
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// I/O error encountered while locating or reading setting sources.
    #[error("Read/Write error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = SettingsError::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }

    #[test]
    fn parsing_error_displays_message() {
        let error = SettingsError::Parsing("invalid value for `port`".to_string());

        assert_eq!(error.to_string(), "Parsing error: invalid value for `port`");
    }

    #[test]
    fn io_error_displays_message() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = SettingsError::Io(io_error);

        assert_eq!(error.to_string(), "Read/Write error: not found");
    }

    #[test]
    fn io_error_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error: SettingsError = io_error.into();

        assert!(matches!(error, SettingsError::Io(_)));
    }
}
