//! Shared test support utilities.

use once_cell::sync::Lazy;

/// Initialises tracing at most once across all integration tests in this process.
///
/// When the `TEST_LOG` environment variable is set, tracing output is enabled at
/// INFO level so test runs can be debugged. Otherwise tracing is not initialised
/// and output stays silent.
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let _ = lib_tracing::init(Some(lib_tracing::Levels::INFO));
    }
});

/// A well-formed Argon2id PHC hash used across integration tests.
///
/// The plaintext password is `"test_password"`.
pub const TEST_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=4096,t=3,p=1$Mzk3NGQ1NDBiZmZiYjhhNDY0YWY3MmRjYjIwNDI2YmE\
     $Bzo73djW0DkCVR4prXhqvWB4ViEdJ54h91mU0LCgqPY";

/// Returns [`lib_settings::RpcSettings`] suitable for integration tests.
///
/// Uses port `0` so the OS assigns an ephemeral port, preventing conflicts
/// when tests run in parallel.
#[allow(dead_code)]
pub fn test_settings() -> lib_settings::RpcSettings {
    lib_settings::RpcSettings {
        host: "127.0.0.1".to_string(),
        port: 0,
        password_hash: TEST_PASSWORD_HASH.to_string(),
        token_secret: "test_secret".to_string(),
    }
}

/// A running test instance of the RPC server.
///
/// Created via [`TestRpcServer::spawn`], which binds to an ephemeral port and runs
/// the server in a background tokio task. The bound address is exposed so
/// tests can connect a gRPC client.
#[allow(dead_code)]
pub struct TestRpcServer {
    pub address: std::net::SocketAddr,
}

impl TestRpcServer {
    /// Spawn a test RPC server on a random port and return the [`TestRpcServer`].
    #[allow(dead_code)]
    pub async fn spawn() -> Self {
        Lazy::force(&TRACING);

        let server = lib_rpc::Server::new(test_settings())
            .await
            .expect("failed to bind test RPC server");

        let bound_address = server.address().expect("failed to get bound address");

        tokio::spawn(server.run());

        Self { address: bound_address }
    }

    /// Connect a [`UtilitiesServiceClient`] to this test server.
    #[allow(dead_code)]
    pub async fn rpc_client(&self) -> lib_rpc::UtilitiesServiceClient<tonic::transport::Channel> {
        lib_rpc::UtilitiesServiceClient::connect(format!("http://{}", self.address))
            .await
            .expect("failed to connect to test RPC server")
    }
}
