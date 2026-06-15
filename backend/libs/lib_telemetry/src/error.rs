//! Telemetry Library Crate Error
//!
//! This module provides error types for the telemetry module.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = TelemetryError::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }
}
