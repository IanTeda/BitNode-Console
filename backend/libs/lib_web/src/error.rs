//! Error types for the `lib_web` crate.

use thiserror::Error;

/// Errors that can occur in the [`crate::HttpServer`].
#[derive(Error, Debug)]
pub enum WebError {
    #[error("Generic error: {0}")]
    Generic(String),

    /// The TCP listener could not bind to the given address.
    #[error("Failed to bind TCP listener on {address}: {source}")]
    Bind {
        address: String,
        #[source]
        source: std::io::Error,
    },

    /// The server encountered an error while serving requests.
    #[error("Server error while serving requests: {0}")]
    Serve(std::io::Error),
}

/// Convenience [`Result`] alias for [`HttpError`].
pub type WebResult<T> = std::result::Result<T, WebError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = WebError::Generic("something went wrong".to_string());
        assert_eq!(error.to_string(), "Generic error: something went wrong");
    }

    #[test]
    fn bind_error_includes_address_and_cause() {
        let source = std::io::Error::new(std::io::ErrorKind::AddrInUse, "address already in use");
        let error = WebError::Bind {
            address: "127.0.0.1:8080".to_string(),
            source,
        };
        let msg = error.to_string();
        assert!(
            msg.contains("127.0.0.1:8080"),
            "expected address in message: {msg}"
        );
        assert!(
            msg.contains("address already in use"),
            "expected cause in message: {msg}"
        );
    }

    #[test]
    fn bind_error_formats_full_message() {
        let source = std::io::Error::new(std::io::ErrorKind::AddrInUse, "address already in use");
        let error = WebError::Bind {
            address: "0.0.0.0:3000".to_string(),
            source,
        };
        assert_eq!(
            error.to_string(),
            "Failed to bind TCP listener on 0.0.0.0:3000: address already in use"
        );
    }

    #[test]
    fn serve_error_displays_message() {
        let source = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let error = WebError::Serve(source);
        assert!(error.to_string().contains("Server error while serving requests"));
    }
}
