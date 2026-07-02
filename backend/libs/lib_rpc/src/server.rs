//-- ./backend/libs/lib_rpc/src/server.rs

//! gRPC server that hosts all RPC services.

use http::header::HeaderName;
use secrecy::SecretString;
use tokio_stream::wrappers::TcpListenerStream;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::services::authentication::{AuthenticationServiceImpl, AuthenticationServiceServer};
use crate::services::journals::{JournalsServiceImpl, JournalsServiceServer};
use crate::services::utilities::{UtilitiesServiceImpl, UtilitiesServiceServer};

/// Combined file descriptor set for all protobuf services in this crate.
///
/// Embedded at compile time from the binary produced by `tonic_build` in `build.rs`.
/// Used by [`tonic_reflection`] to serve schema information to reflection-capable clients
/// (e.g. grpcurl, Postman, BloomRPC) without requiring out-of-band `.proto` files.
const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/bitnode_console_v1_descriptor.bin"
));

/// gRPC server bound to a TCP listener.
///
/// Construction and serving are split so that integration tests can bind to
/// port `0`, read back the assigned port via [`address`](Self::address),
/// and then spawn [`run`](Self::run) in a background task.
#[derive(Debug)]
pub struct Server {
    /// The TCP listener that serves incoming gRPC requests.
    listener: tokio::net::TcpListener,

    /// The resolved settings for the gRPC server.
    settings: lib_settings::Settings,
}

impl Server {
    /// Bind the gRPC server to the address derived from `settings`.
    ///
    /// The socket address is resolved from `settings.host` and `settings.port`.
    /// Use port `0` to let the OS assign an ephemeral port; read it back with
    /// [`address`](Self::address).
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Config`] if `settings.host` is not a valid IP address.
    /// Returns [`crate::Error::Transport`] if the TCP listener cannot bind to the address.
    #[tracing::instrument(skip(settings))]
    pub async fn new(settings: lib_settings::Settings) -> crate::Result<Self> {
        let address =
            settings.rpc.socket_address().map_err(|e| crate::Error::Config(e.to_string()))?;

        let listener = tokio::net::TcpListener::bind(address)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        tracing::debug!("RPC server bound to {}", listener.local_addr().unwrap());

        Ok(Self { listener, settings })
    }

    /// Returns the local socket address the server is bound to.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Transport`] if the OS cannot read back the bound address.
    #[tracing::instrument(skip(self))]
    pub fn address(&self) -> crate::Result<std::net::SocketAddr> {
        self.listener.local_addr().map_err(|e| crate::Error::Transport(e.to_string()))
    }

    /// Start serving gRPC requests.
    ///
    /// Registers all RPC services and serves until the process is shut down or
    /// an unrecoverable transport error occurs.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Config`] if `settings.password_hash` is not a valid PHC string.
    /// Returns [`crate::Error::Transport`] if the server encounters a transport error while serving.
    #[tracing::instrument(skip(self))]
    pub async fn run(self) -> crate::Result<()> {
        //--- Check that password_hash is configured, else stop the server
        if self.settings.rpc.password_hash().is_empty() {
            return Err(crate::Error::Config(
                "rpc.password_hash is not configured".to_string(),
            ));
        }

        //--- Build interceptors
        let allowed_ips_interceptor = crate::interceptors::AllowedIpsInterceptor::new(
            self.settings.rpc.allowed_ips().to_vec(),
        );
        let allowed_ips_interceptor = tonic::service::interceptor(allowed_ips_interceptor);

        let access_token_interceptor = crate::interceptors::AccessTokenInterceptor::new(
            self.settings.rpc.token_secret().into(),
        );

        // --- CORS Layer
        // Create a new CORS layer for gRPC-Web browser clients.
        let cors_layer = CorsLayer::new()
            .allow_origin(AllowOrigin::mirror_request())
            .allow_methods([http::Method::POST])
            .allow_headers([
                HeaderName::from_static("access_token"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
                HeaderName::from_static("x-grpc-web"),
            ])
            .expose_headers([
                HeaderName::from_static("grpc-status"),
                HeaderName::from_static("grpc-message"),
            ]);

        // --- gRPC-Web Layer
        // This layer is used to serve gRPC-Web browser clients.
        let grpc_web_layer = tonic_web::GrpcWebLayer::new();

        // --- Authentication RPC Service
        // Build a new authentication service (no access-token required to log in).
        let password_hash = lib_auth::PasswordHash::try_from(self.settings.rpc.password_hash())
            .map_err(|e| crate::Error::Config(e.to_string()))?;
        let token_secret = SecretString::from(self.settings.rpc.token_secret());
        let auth_service = AuthenticationServiceServer::new(AuthenticationServiceImpl::new(
            password_hash,
            token_secret,
        ));
        tracing::debug!("Authentication service registered");

        // --- Utilities RPC Service
        // Build a new utilities service, protected by the access-token interceptor.
        let utilities_service = UtilitiesServiceServer::with_interceptor(
            UtilitiesServiceImpl::default(),
            access_token_interceptor,
        );
        tracing::debug!("Utilities service registered");

        // --- Journals RPC Service
        // Build a new journals service, protected by a separate access-token interceptor.
        let journals_access_token_interceptor = crate::interceptors::AccessTokenInterceptor::new(
            self.settings.rpc.token_secret().into(),
        );
        let journals_service = JournalsServiceServer::with_interceptor(
            JournalsServiceImpl::default(),
            journals_access_token_interceptor,
        );
        tracing::debug!("Journals service registered");

        // --- Reflection RPC Service
        // Build the reflection service to serve schema information to reflection-capable clients.
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| crate::Error::Config(e.to_string()))?;
        tracing::debug!("Reflection service registered");

        //--- Set up the server address and incoming stream
        let addr = self.address()?;
        let incoming_stream = TcpListenerStream::new(self.listener);

        tracing::info!("RPC server listening on rpc://{addr}");

        // Build and serve the gRPC server.
        tonic::transport::Server::builder()
            .accept_http1(true)
            .layer(allowed_ips_interceptor)
            .layer(cors_layer)
            .layer(grpc_web_layer)
            .add_service(auth_service)
            .add_service(utilities_service)
            .add_service(journals_service)
            .add_service(reflection_service)
            .serve_with_incoming(incoming_stream)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_protos::utilities::utilities_service_client::UtilitiesServiceClient;
    use secrecy::SecretString;
    use std::sync::OnceLock;

    /// A valid Argon2id hash of `"test_password"` computed once per test process.
    static TEST_HASH: OnceLock<lib_auth::PasswordHash> = OnceLock::new();

    fn test_hash() -> &'static lib_auth::PasswordHash {
        TEST_HASH.get_or_init(|| {
            lib_auth::PasswordHash::from_password(&SecretString::from("test_password"))
                .expect("test hash must compute")
        })
    }

    /// Base settings: binds to `127.0.0.1:0` so the OS assigns an ephemeral port.
    fn settings() -> lib_settings::Settings {
        lib_settings::Settings {
            rpc: lib_settings::RpcSettings {
                host: "127.0.0.1".to_string(),
                port: 0,
                password_hash: test_hash().as_ref().to_string(),
                token_secret: "test_secret".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn settings_with_host(host: &str) -> lib_settings::Settings {
        lib_settings::Settings {
            rpc: lib_settings::RpcSettings {
                host: host.to_string(),
                ..settings().rpc
            },
            ..settings()
        }
    }

    fn settings_with_port(port: u16) -> lib_settings::Settings {
        lib_settings::Settings {
            rpc: lib_settings::RpcSettings {
                port,
                ..settings().rpc
            },
            ..settings()
        }
    }

    /// Generate a valid access token signed with the test secret.
    fn valid_access_token() -> String {
        lib_auth::AccessToken::new(&SecretString::from("test_secret"))
            .expect("test access token must generate")
            .to_string()
    }

    // --- new ---

    #[tokio::test]
    async fn new_binds_to_ephemeral_port() {
        let result = Server::new(settings()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn new_fails_with_invalid_host() {
        let result = Server::new(settings_with_host("not-an-ip")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn new_invalid_host_returns_config_error() {
        let err = Server::new(settings_with_host("not-an-ip")).await.unwrap_err();

        assert!(
            matches!(err, crate::Error::Config(_)),
            "expected Config variant, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn new_fails_when_port_already_in_use() {
        let first = Server::new(settings()).await.unwrap();
        let taken_port = first.address().unwrap().port();

        let result = Server::new(settings_with_port(taken_port)).await;

        assert!(result.is_err(), "binding to a taken port should fail");
    }

    #[tokio::test]
    async fn new_port_conflict_returns_transport_error() {
        let first = Server::new(settings()).await.unwrap();
        let taken_port = first.address().unwrap().port();

        let err = Server::new(settings_with_port(taken_port)).await.unwrap_err();

        assert!(
            matches!(err, crate::Error::Transport(_)),
            "expected Transport variant, got: {err:?}"
        );
    }

    // --- address ---

    #[tokio::test]
    async fn address_returns_localhost() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();

        assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
    }

    #[tokio::test]
    async fn address_returns_os_assigned_port_when_port_zero() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();

        assert_ne!(addr.port(), 0, "OS should assign a non-zero port");
    }

    #[tokio::test]
    async fn two_servers_bind_to_different_ports() {
        let a = Server::new(settings()).await.unwrap();
        let b = Server::new(settings()).await.unwrap();

        assert_ne!(a.address().unwrap().port(), b.address().unwrap().port());
    }

    // --- debug ---

    #[tokio::test]
    async fn debug_format_includes_struct_name() {
        let server = Server::new(settings()).await.unwrap();

        assert!(format!("{server:?}").contains("Server"));
    }

    // --- run: utilities service ---

    #[tokio::test]
    async fn run_responds_to_ping() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let mut client = UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();
        let mut request = tonic::Request::new(crate::services::utilities::PingRequest {});
        request
            .metadata_mut()
            .insert("access_token", valid_access_token().parse().unwrap());
        let response = client.ping(request).await.unwrap();

        assert_eq!(response.into_inner().pong, "Pong...");
    }

    #[tokio::test]
    async fn run_ping_is_idempotent() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let mut client = UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();

        for _ in 0..3 {
            let mut request = tonic::Request::new(crate::services::utilities::PingRequest {});
            request
                .metadata_mut()
                .insert("access_token", valid_access_token().parse().unwrap());
            let response = client.ping(request).await.unwrap();
            assert_eq!(response.into_inner().pong, "Pong...");
        }
    }

    #[tokio::test]
    async fn run_ping_without_access_token_returns_unauthenticated() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let mut client = UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();
        let err = client
            .ping(tonic::Request::new(
                crate::services::utilities::PingRequest {},
            ))
            .await
            .unwrap_err();

        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn run_ping_with_invalid_access_token_returns_unauthenticated() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let mut client = UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();
        let mut request = tonic::Request::new(crate::services::utilities::PingRequest {});
        request
            .metadata_mut()
            .insert("access_token", "not.a.valid.jwt".parse().unwrap());
        let err = client.ping(request).await.unwrap_err();

        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // --- run: CORS ---

    #[tokio::test]
    async fn cors_preflight_mirrors_request_origin() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let response = reqwest::Client::new()
            .request(
                reqwest::Method::OPTIONS,
                format!("http://{addr}/bitnode_console.v1.utilities.UtilitiesService/Ping"),
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
    async fn cors_preflight_allows_grpc_web_headers() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        let response = reqwest::Client::new()
            .request(
                reqwest::Method::OPTIONS,
                format!("http://{addr}/bitnode_console.v1.utilities.UtilitiesService/Ping"),
            )
            .header("Origin", "http://localhost:8086")
            .header("Access-Control-Request-Method", "POST")
            .header(
                "Access-Control-Request-Headers",
                "content-type,x-grpc-web,access_token",
            )
            .send()
            .await
            .unwrap();

        // Preflight responses advertise which request headers are allowed.
        // Exposed response headers (grpc-status, grpc-message) are only sent
        // on actual responses, not on the preflight OPTIONS response.
        let allowed = response
            .headers()
            .get("access-control-allow-headers")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            allowed.contains("access_token"),
            "access_token not in allowed headers: {allowed}"
        );
        assert!(
            allowed.contains("authorization"),
            "authorization not in allowed headers: {allowed}"
        );
        assert!(
            allowed.contains("content-type"),
            "content-type not in allowed headers: {allowed}"
        );
        assert!(
            allowed.contains("x-grpc-web"),
            "x-grpc-web not in allowed headers: {allowed}"
        );
    }

    // --- run: gRPC-Web ---

    #[tokio::test]
    async fn grpc_web_ping_returns_ok_with_grpc_web_content_type() {
        let server = Server::new(settings()).await.unwrap();
        let addr = server.address().unwrap();
        tokio::spawn(server.run());

        // gRPC-Web frame: 0x00 (no compression) + 4-byte big-endian length + payload.
        // PingRequest has no fields, so length = 0.
        let frame: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00];

        let response = reqwest::Client::new()
            .post(format!(
                "http://{addr}/bitnode_console.v1.utilities.UtilitiesService/Ping"
            ))
            .header("Content-Type", "application/grpc-web")
            .header("x-grpc-web", "1")
            .header("access_token", valid_access_token())
            .body(frame)
            .send()
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "expected 200 OK, got {}",
            response.status()
        );

        let content_type =
            response.headers().get("content-type").unwrap().to_str().unwrap().to_string();
        assert!(
            content_type.starts_with("application/grpc-web"),
            "expected grpc-web content type, got: {content_type}"
        );

        // Consume the body — reqwest's raw HTTP client does not reassemble
        // tonic-web's chunked gRPC framing; status + content-type confirm the
        // gRPC-Web layer is active.
        let _ = response.bytes().await;
    }

    // --- run: invalid settings ---

    #[tokio::test]
    async fn run_with_empty_password_hash_returns_config_error() {
        let server = Server::new(lib_settings::Settings {
            rpc: lib_settings::RpcSettings {
                password_hash: String::new(),
                ..settings().rpc
            },
            ..settings()
        })
        .await
        .unwrap();

        let err = server.run().await.unwrap_err();

        assert!(
            matches!(err, crate::Error::Config(_)),
            "expected Config variant, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn run_with_malformed_password_hash_returns_config_error() {
        let server = Server::new(lib_settings::Settings {
            rpc: lib_settings::RpcSettings {
                password_hash: "not-a-phc-string".to_string(),
                ..settings().rpc
            },
            ..settings()
        })
        .await
        .unwrap();

        let err = server.run().await.unwrap_err();

        assert!(
            matches!(err, crate::Error::Config(_)),
            "expected Config variant, got: {err:?}"
        );
    }
}
