//! Error types for the server crate.
//!
//! Defines [`ServerError`], the top-level error type returned by fallible
//! operations in this crate, along with its associated [`ServerResult`]
//! alias (re-exported via the crate prelude).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    Generic(String),

    /// Settings loading or validating application errors.
    #[error("Settings error: {0}")]
    Settings(#[from] lib_settings::SettingsError),

    /// Telemetry initialisation errors.
    #[error("Telemetry error: {0}")]
    Telemetry(#[from] lib_telemetry::TelemetryError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = ServerError::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }
}
