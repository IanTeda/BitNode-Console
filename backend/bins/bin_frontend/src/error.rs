//! Error types for the `bin_frontend` binary.

/// Top-level error type for the web-frontend-only server.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic error {0}")]
    Generic(String),

    /// Settings loading or validating errors.
    #[error("Settings error: {0}")]
    Settings(#[from] lib_settings::Error),

    /// Telemetry initialisation errors.
    #[error("Tracing error: {0}")]
    Tracing(#[from] lib_tracing::Error),

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

    #[test]
    fn settings_error_displays_message() {
        let settings_error = lib_settings::Error::Generic("bad config".to_string());
        let error = Error::Settings(settings_error);
        assert_eq!(error.to_string(), "Settings error: Generic error bad config");
    }

    #[test]
    fn settings_error_from_conversion() {
        let settings_error = lib_settings::Error::Parsing("invalid port".to_string());
        let error: Error = settings_error.into();
        assert!(matches!(error, Error::Settings(_)));
    }

    #[test]
    fn tracing_error_displays_message() {
        let tracing_error = lib_tracing::Error::Generic("init failed".to_string());
        let error = Error::Tracing(tracing_error);
        assert_eq!(error.to_string(), "Tracing error: Generic error init failed");
    }

    #[test]
    fn tracing_error_from_conversion() {
        let tracing_error = lib_tracing::Error::Generic("subscriber error".to_string());
        let error: Error = tracing_error.into();
        assert!(matches!(error, Error::Tracing(_)));
    }

    #[test]
    fn web_error_displays_message() {
        let web_error = lib_web::Error::Generic("connection refused".to_string());
        let error = Error::Web(web_error);
        assert_eq!(error.to_string(), "Web error: Generic error: connection refused");
    }

    #[test]
    fn web_error_from_conversion() {
        let web_error = lib_web::Error::Generic("timeout".to_string());
        let error: Error = web_error.into();
        assert!(matches!(error, Error::Web(_)));
    }

    #[test]
    fn debug_format_includes_variant_name() {
        let generic = Error::Generic("x".to_string());
        assert!(format!("{generic:?}").contains("Generic"));

        let settings = Error::Settings(lib_settings::Error::Generic("x".to_string()));
        assert!(format!("{settings:?}").contains("Settings"));

        let tracing = Error::Tracing(lib_tracing::Error::Generic("x".to_string()));
        assert!(format!("{tracing:?}").contains("Tracing"));

        let web = Error::Web(lib_web::Error::Generic("x".to_string()));
        assert!(format!("{web:?}").contains("Web"));
    }
}
