//-- ./backend/libs/lib_settings/src/error.rs

//! Settings Library Crate Error
//!
//! This module provides error types for the settings module.

#[derive(thiserror::Error, Debug)]
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

    #[test]
    fn debug_format_includes_variant_name() {
        assert!(format!("{:?}", SettingsError::Generic("x".to_string())).contains("Generic"));
        assert!(format!("{:?}", SettingsError::Parsing("x".to_string())).contains("Parsing"));
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        assert!(format!("{:?}", SettingsError::Io(io_err)).contains("Io"));
    }

    #[test]
    fn io_error_from_conversion_preserves_error_kind() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error: SettingsError = io_error.into();

        if let SettingsError::Io(inner) = error {
            assert_eq!(inner.kind(), std::io::ErrorKind::PermissionDenied);
        } else {
            panic!("expected Io variant");
        }
    }

    #[test]
    fn io_error_source_exposes_original_error() {
        use std::error::Error;

        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let error = SettingsError::Io(io_error);

        assert!(error.source().is_some());
    }
}
