//-- ./backend/libs/lib_web/src/server.rs

//! HTTP server for the `BitNode` Console backend.

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

/// Directory containing the compiled frontend assets, embedded at compile time.
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

/// HTTP server that listens on a configured host and port.
///
/// Serves the compiled frontend as a single-page application, falling back
/// to `index.html` for paths not matched by a static file to support
/// client-side routing.
pub struct HttpServer {
    host: String,
    port: u16,
}

impl HttpServer {
    /// Create a new [`HttpServer`] bound to `host` and `port`.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        tracing::info!("Creating new http server");
        Self {
            host: host.into(),
            port,
        }
    }

    /// Returns the `"host:port"` address string this server will bind to.
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Start the HTTP server and begin serving requests.
    ///
    /// Binds a TCP listener to the configured address and serves the compiled
    /// frontend as a single-page application.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Bind`] if the TCP listener cannot bind to the
    /// configured address (e.g. port already in use).
    /// Returns [`HttpError::Serve`] if the server encounters an error while
    /// serving requests.
    pub async fn run(&self) -> crate::Result<()> {
        let address = self.address();
        tracing::info!("Starting web server on http://{address}");

        let index_html = format!("{ASSETS_DIR}/index.html");
        let static_files = ServeDir::new(ASSETS_DIR).not_found_service(ServeFile::new(&index_html));
        let app = Router::new().fallback_service(static_files);

        let listener =
            tokio::net::TcpListener::bind(&address)
                .await
                .map_err(|source| crate::Error::Bind {
                    address: address.clone(),
                    source,
                })?;

        axum::serve(listener, app).await.map_err(crate::Error::Serve)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_str_slice() {
        let server = HttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.address(), "127.0.0.1:8080");
    }

    #[test]
    fn new_accepts_owned_string() {
        let host = "localhost".to_string();
        let server = HttpServer::new(host, 3000);
        assert_eq!(server.address(), "localhost:3000");
    }

    #[test]
    fn address_formats_host_colon_port() {
        let server = HttpServer::new("0.0.0.0", 8090);
        assert_eq!(server.address(), "0.0.0.0:8090");
    }

    #[test]
    fn address_with_port_zero() {
        let server = HttpServer::new("127.0.0.1", 0);
        assert_eq!(server.address(), "127.0.0.1:0");
    }

    #[test]
    fn address_with_max_port() {
        let server = HttpServer::new("127.0.0.1", u16::MAX);
        assert_eq!(server.address(), format!("127.0.0.1:{}", u16::MAX));
    }

    #[test]
    fn address_with_ipv6_host() {
        let server = HttpServer::new("::1", 8080);
        assert_eq!(server.address(), "::1:8080");
    }

    #[tokio::test]
    async fn run_returns_error_for_invalid_address() {
        let server = HttpServer::new("999.999.999.999", 8080);
        let result = server.run().await;
        assert!(result.is_err());
    }
}
