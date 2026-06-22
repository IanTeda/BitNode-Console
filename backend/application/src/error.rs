//! Application error types.
//!
//! Defines [`ApplicationError`], the top-level error type returned by fallible
//! operations in this crate.

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    Generic(String),

    /// Settings loading or validating application errors.
    #[error("Settings error: {0}")]
    Settings(#[from] lib_settings::Error),

    /// Telemetry initialisation errors.
    #[error("Telemetry error: {0}")]
    Telemetry(#[from] lib_tracing::Error),

    /// Web server errors.
    #[error("Web error: {0}")]
    Web(#[from] lib_web::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = Error::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Generic error something went wrong");
    }
}
