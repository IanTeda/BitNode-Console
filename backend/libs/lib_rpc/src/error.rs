//! Error types for the `lib_rpc` crate.

/// Errors that can occur within the `lib_rpc` crate.
///
/// The enum is marked [`non_exhaustive`] so that adding new variants in future
/// releases does not constitute a breaking change for downstream consumers.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A transport-level failure, such as a failed connection or dropped stream.
    #[error("Transport error: {0}")]
    Transport(String),

    /// The server did not respond within the allowed time.
    #[error("Request timed out")]
    Timeout,

    /// A serialisation or deserialisation failure.
    #[error("Serialisation error: {0}")]
    Serialisation(String),

    /// Invalid or missing configuration prevented the RPC client from starting.
    #[error("Configuration error: {0}")]
    Config(String),

    /// The request was rejected because a token was missing, malformed, expired,
    /// or of the wrong type.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// The caller's identity is known but they are not permitted to access the resource.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Catch-all for errors that do not fit a more specific variant.
    #[error("RPC error: {0}")]
    Generic(String),
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::Authentication(msg) => tonic::Status::unauthenticated(msg),
            Error::PermissionDenied(msg) => tonic::Status::permission_denied(msg),
            other => tonic::Status::internal(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Display formatting ---------------------------------------------------

    #[test]
    fn transport_error_displays_message() {
        let error = Error::Transport("connection refused".to_string());
        assert_eq!(error.to_string(), "Transport error: connection refused");
    }

    #[test]
    fn timeout_error_displays_message() {
        let error = Error::Timeout;
        assert_eq!(error.to_string(), "Request timed out");
    }

    #[test]
    fn serialisation_error_displays_message() {
        let error = Error::Serialisation("unexpected end of input".to_string());
        assert_eq!(
            error.to_string(),
            "Serialisation error: unexpected end of input"
        );
    }

    #[test]
    fn config_error_displays_message() {
        let error = Error::Config("missing endpoint".to_string());
        assert_eq!(error.to_string(), "Configuration error: missing endpoint");
    }

    #[test]
    fn authentication_error_displays_message() {
        let error = Error::Authentication("token expired".to_string());
        assert_eq!(error.to_string(), "Authentication error: token expired");
    }

    #[test]
    fn permission_denied_error_displays_message() {
        let error = Error::PermissionDenied("ip not allowed".to_string());
        assert_eq!(error.to_string(), "Permission denied: ip not allowed");
    }

    #[test]
    fn generic_error_displays_message() {
        let error = Error::Generic("something went wrong".to_string());
        assert_eq!(error.to_string(), "RPC error: something went wrong");
    }

    // --- Debug formatting ----------------------------------------------------

    #[test]
    fn debug_format_includes_variant_and_payload() {
        let error = Error::Transport("ECONNREFUSED".to_string());
        let debug = format!("{error:?}");
        assert!(debug.contains("Transport"));
        assert!(debug.contains("ECONNREFUSED"));
    }

    // --- std::error::Error trait ---------------------------------------------

    #[test]
    fn rpc_error_implements_std_error() {
        // Compile-time assertion: RpcError must satisfy the std::error::Error bound.
        fn assert_std_error<E: std::error::Error>() {}
        assert_std_error::<Error>();
    }

    #[test]
    fn string_variants_have_no_error_source() {
        // Variants without #[source] must return None from source().
        let cases: &[Error] = &[
            Error::Transport("t".to_string()),
            Error::Timeout,
            Error::Serialisation("s".to_string()),
            Error::Config("c".to_string()),
            Error::Authentication("a".to_string()),
            Error::PermissionDenied("p".to_string()),
            Error::Generic("g".to_string()),
        ];
        for error in cases {
            assert!(
                std::error::Error::source(error).is_none(),
                "{error:?} unexpectedly has a source"
            );
        }
    }

    // --- RpcResult type alias ------------------------------------------------

    #[test]
    fn rpc_result_ok_carries_value() {
        let result: crate::Result<u32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn rpc_result_err_carries_error() {
        let result: crate::Result<u32> = Err(Error::Timeout);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Request timed out");
    }

    #[test]
    fn rpc_result_propagates_with_question_mark() {
        fn fallible() -> crate::Result<u32> {
            let inner: crate::Result<u32> = Err(Error::Generic("fail".to_string()));
            let value = inner?;
            Ok(value)
        }

        let result = fallible();
        assert!(result.is_err());
    }

    // --- From<Error> for tonic::Status ---

    #[test]
    fn authentication_error_converts_to_unauthenticated_status() {
        let status = tonic::Status::from(Error::Authentication("bad token".to_string()));
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn authentication_error_preserves_message_in_status() {
        let status = tonic::Status::from(Error::Authentication("bad token".to_string()));
        assert_eq!(status.message(), "bad token");
    }

    #[test]
    fn permission_denied_error_converts_to_permission_denied_status() {
        let status = tonic::Status::from(Error::PermissionDenied("ip not allowed".to_string()));
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }

    #[test]
    fn permission_denied_error_preserves_message_in_status() {
        let status = tonic::Status::from(Error::PermissionDenied("ip not allowed".to_string()));
        assert_eq!(status.message(), "ip not allowed");
    }

    #[test]
    fn non_authentication_errors_convert_to_internal_status() {
        let cases = [
            Error::Transport("t".to_string()),
            Error::Timeout,
            Error::Config("c".to_string()),
            Error::Generic("g".to_string()),
        ];
        for error in cases {
            let status = tonic::Status::from(error);
            assert_eq!(
                status.code(),
                tonic::Code::Internal,
                "expected Internal for {status:?}"
            );
        }
    }
}
