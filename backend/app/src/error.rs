//! Application error types.
//!
//! Defines [`ApplicationError`], the top-level error type returned by fallible
//! operations in this crate.

/// The BitNode Console backend application error type.
///
/// This error type has uses the library crate error types as its underlying error types.
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

    /// RPC server errors.
    #[error("RPC error: {0}")]
    Rpc(#[from] lib_rpc::Error),
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
    fn telemetry_error_displays_message() {
        let tracing_error = lib_tracing::Error::Generic("init failed".to_string());
        let error = Error::Telemetry(tracing_error);

        assert_eq!(error.to_string(), "Telemetry error: Generic error init failed");
    }

    #[test]
    fn telemetry_error_from_conversion() {
        let tracing_error = lib_tracing::Error::Generic("subscriber error".to_string());
        let error: Error = tracing_error.into();

        assert!(matches!(error, Error::Telemetry(_)));
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
    fn web_bind_error_displays_nested_message() {
        let io_error = std::io::Error::new(std::io::ErrorKind::AddrInUse, "address in use");
        let web_error = lib_web::Error::Bind {
            address: "0.0.0.0:3000".to_string(),
            source: io_error,
        };
        let error = Error::Web(web_error);

        let msg = error.to_string();
        assert!(msg.contains("0.0.0.0:3000"), "expected address in: {msg}");
        assert!(msg.contains("address in use"), "expected cause in: {msg}");
    }

    #[test]
    fn settings_error_source_exposes_underlying_error() {
        let settings_error = lib_settings::Error::Generic("oops".to_string());
        let error = Error::Settings(settings_error);

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn telemetry_error_source_exposes_underlying_error() {
        let tracing_error = lib_tracing::Error::Generic("oops".to_string());
        let error = Error::Telemetry(tracing_error);

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn web_error_source_exposes_underlying_error() {
        let web_error = lib_web::Error::Generic("oops".to_string());
        let error = Error::Web(web_error);

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn debug_format_includes_variant_name() {
        let generic = Error::Generic("x".to_string());
        assert!(format!("{generic:?}").contains("Generic"));

        let settings = Error::Settings(lib_settings::Error::Generic("x".to_string()));
        assert!(format!("{settings:?}").contains("Settings"));

        let telemetry = Error::Telemetry(lib_tracing::Error::Generic("x".to_string()));
        assert!(format!("{telemetry:?}").contains("Telemetry"));

        let web = Error::Web(lib_web::Error::Generic("x".to_string()));
        assert!(format!("{web:?}").contains("Web"));
    }
}
