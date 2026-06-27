//-- ./backend/libs/lib_rpc/src/server.rs

//! gRPC server that hosts all RPC services.

use http::header::HeaderName;
use tokio_stream::wrappers::TcpListenerStream;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::services::{UtilitiesServiceImpl, UtilitiesServiceServer};

/// gRPC server bound to a TCP listener.
///
/// Construction and serving are split so that integration tests can bind to
/// port `0`, read back the assigned port via [`address`](Self::address),
/// and then spawn [`run`](Self::run) in a background task.
#[derive(Debug)]
pub struct Server {
    /// The TCP listener that serves incoming gRPC requests.
    listener: tokio::net::TcpListener,
}

impl Server {
    /// Bind the gRPC server to the given socket address.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Transport`] if the TCP listener cannot bind.
    pub async fn new(address: std::net::SocketAddr) -> crate::Result<Self> {
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        tracing::debug!("RPC server bound to {}", listener.local_addr().unwrap());

        Ok(Self { listener })
    }

    /// Returns the local address the server is bound to.
    pub fn address(&self) -> crate::Result<std::net::SocketAddr> {
        self.listener.local_addr().map_err(|e| crate::Error::Transport(e.to_string()))
    }

    /// Start serving gRPC requests.
    ///
    /// Registers all RPC services and serves until the process is shut down.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Transport`] if the server encounters an error
    /// while serving.
    pub async fn run(self) -> crate::Result<()> {
        let addr = self.address()?;
        let incoming = TcpListenerStream::new(self.listener);

        let utilities = UtilitiesServiceServer::new(UtilitiesServiceImpl::default());

        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::mirror_request())
            .allow_methods([http::Method::POST])
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("x-grpc-web"),
            ])
            .expose_headers([
                HeaderName::from_static("grpc-status"),
                HeaderName::from_static("grpc-message"),
            ]);

        tracing::info!("RPC server listening on rpc://{addr}");

        tonic::transport::Server::builder()
            .accept_http1(true)
            .layer(cors)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(utilities)
            .serve_with_incoming(incoming)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn localhost(port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], port))
    }

    #[tokio::test]
    async fn new_binds_to_ephemeral_port() {
        let server = Server::new(localhost(0)).await;

        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn address_returns_bound_address() {
        let server = Server::new(localhost(0)).await.unwrap();
        let addr = server.address().unwrap();

        assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_ne!(addr.port(), 0, "OS should assign a real port");
    }

    #[tokio::test]
    async fn new_fails_on_invalid_address() {
        let result = Server::new(std::net::SocketAddr::from(([255, 255, 255, 255], 1))).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn new_error_is_transport_variant() {
        let err = Server::new(std::net::SocketAddr::from(([255, 255, 255, 255], 1)))
            .await
            .unwrap_err();

        assert!(
            matches!(err, crate::Error::Transport(_)),
            "expected Transport variant, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn two_servers_bind_to_different_ports() {
        let server_a = Server::new(localhost(0)).await.unwrap();
        let server_b = Server::new(localhost(0)).await.unwrap();

        assert_ne!(
            server_a.address().unwrap().port(),
            server_b.address().unwrap().port(),
        );
    }

    #[tokio::test]
    async fn duplicate_port_bind_fails() {
        let first = Server::new(localhost(0)).await.unwrap();
        let taken_port = first.address().unwrap().port();

        let second = Server::new(localhost(taken_port)).await;

        assert!(
            second.is_err(),
            "binding to an already-taken port should fail"
        );
    }

    #[tokio::test]
    async fn debug_format_includes_struct_name() {
        let server = Server::new(localhost(0)).await.unwrap();

        assert!(format!("{server:?}").contains("Server"));
    }

    #[tokio::test]
    async fn run_serves_and_responds_to_ping() {
        let server = Server::new(localhost(0)).await.unwrap();
        let addr = server.address().unwrap();

        tokio::spawn(server.run());

        let mut client =
            crate::UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();

        let response = client
            .ping(tonic::Request::new(crate::generated_protos::PingRequest {}))
            .await
            .unwrap();

        assert_eq!(response.into_inner().pong, "Pong...");
    }

    #[tokio::test]
    async fn run_ping_is_idempotent() {
        let server = Server::new(localhost(0)).await.unwrap();
        let addr = server.address().unwrap();

        tokio::spawn(server.run());

        let mut client =
            crate::UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();

        for _ in 0..3 {
            let response = client
                .ping(tonic::Request::new(crate::generated_protos::PingRequest {}))
                .await
                .unwrap();

            assert_eq!(response.into_inner().pong, "Pong...");
        }
    }

    #[tokio::test]
    async fn cors_preflight_returns_allow_origin() {
        let server = Server::new(localhost(0)).await.unwrap();
        let addr = server.address().unwrap();

        tokio::spawn(server.run());

        let client = reqwest::Client::new();
        let response = client
            .request(
                reqwest::Method::OPTIONS,
                format!("http://{addr}/bitnode_console.v1.UtilitiesService/Ping"),
            )
            .header("Origin", "http://localhost:8086")
            .header("Access-Control-Request-Method", "POST")
            .header("Access-Control-Request-Headers", "content-type,x-grpc-web")
            .send()
            .await
            .unwrap();

        assert_eq!(
            response.headers().get("access-control-allow-origin").unwrap(),
            "http://localhost:8086",
        );
    }

    #[tokio::test]
    async fn grpc_web_ping_returns_response() {
        let server = Server::new(localhost(0)).await.unwrap();
        let addr = server.address().unwrap();

        tokio::spawn(server.run());

        // gRPC-Web frame: 0x00 (no compression) + 4-byte big-endian length + protobuf payload.
        // PingRequest is empty, so length is 0.
        let grpc_web_body: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00];

        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "http://{addr}/bitnode_console.v1.UtilitiesService/Ping"
            ))
            .header("Content-Type", "application/grpc-web")
            .header("x-grpc-web", "1")
            .body(grpc_web_body)
            .send()
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "expected 200 OK, got {}",
            response.status()
        );

        let content_type = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(
            content_type.starts_with("application/grpc-web"),
            "expected grpc-web content type, got: {content_type}"
        );

        let body = response.bytes().await.unwrap();
        assert!(!body.is_empty(), "response body should not be empty");
    }
}
