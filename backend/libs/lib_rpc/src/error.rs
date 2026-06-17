//! Error types for the `lib_rpc` crate.

use thiserror::Error;

/// Errors that can occur within the `lib_rpc` crate.
///
/// The enum is marked [`non_exhaustive`] so that adding new variants in future
/// releases does not constitute a breaking change for downstream consumers.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum RpcError {
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

    /// Catch-all for errors that do not fit a more specific variant.
    #[error("RPC error: {0}")]
    Generic(String),
}

/// Convenience [`Result`] alias using [`RpcError`] as the error type.
pub type RpcResult<T> = std::result::Result<T, RpcError>;

#[cfg(test)]
mod tests {
    use super::*;

    // --- Display formatting ---------------------------------------------------

    #[test]
    fn transport_error_displays_message() {
        let error = RpcError::Transport("connection refused".to_string());
        assert_eq!(error.to_string(), "Transport error: connection refused");
    }

    #[test]
    fn timeout_error_displays_message() {
        let error = RpcError::Timeout;
        assert_eq!(error.to_string(), "Request timed out");
    }

    #[test]
    fn serialisation_error_displays_message() {
        let error = RpcError::Serialisation("unexpected end of input".to_string());
        assert_eq!(
            error.to_string(),
            "Serialisation error: unexpected end of input"
        );
    }

    #[test]
    fn config_error_displays_message() {
        let error = RpcError::Config("missing endpoint".to_string());
        assert_eq!(error.to_string(), "Configuration error: missing endpoint");
    }

    #[test]
    fn generic_error_displays_message() {
        let error = RpcError::Generic("something went wrong".to_string());
        assert_eq!(error.to_string(), "RPC error: something went wrong");
    }

    // --- Debug formatting ----------------------------------------------------

    #[test]
    fn debug_format_includes_variant_and_payload() {
        let error = RpcError::Transport("ECONNREFUSED".to_string());
        let debug = format!("{error:?}");
        assert!(debug.contains("Transport"));
        assert!(debug.contains("ECONNREFUSED"));
    }

    // --- std::error::Error trait ---------------------------------------------

    #[test]
    fn rpc_error_implements_std_error() {
        // Compile-time assertion: RpcError must satisfy the std::error::Error bound.
        fn assert_std_error<E: std::error::Error>() {}
        assert_std_error::<RpcError>();
    }

    #[test]
    fn string_variants_have_no_error_source() {
        // Variants without #[source] must return None from source().
        let cases: &[RpcError] = &[
            RpcError::Transport("t".to_string()),
            RpcError::Timeout,
            RpcError::Serialisation("s".to_string()),
            RpcError::Config("c".to_string()),
            RpcError::Generic("g".to_string()),
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
        let result: RpcResult<u32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn rpc_result_err_carries_error() {
        let result: RpcResult<u32> = Err(RpcError::Timeout);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Request timed out");
    }

    #[test]
    fn rpc_result_propagates_with_question_mark() {
        fn fallible() -> RpcResult<u32> {
            let inner: RpcResult<u32> = Err(RpcError::Generic("fail".to_string()));
            let value = inner?;
            Ok(value)
        }

        let result = fallible();
        assert!(result.is_err());
    }
}
