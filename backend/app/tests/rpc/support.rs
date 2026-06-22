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

/// A running test instance of the RPC server.
///
/// Created via [`TestApp::spawn`], which binds to an ephemeral port and runs
/// the server in a background tokio task. The bound address is exposed so
/// tests can connect a gRPC client.
#[allow(dead_code)]
pub struct TestRpcServer {
    pub address: std::net::SocketAddr,
}

impl TestRpcServer {
    /// Spawn a test RPC server on a random port and return the [`TestApp`].
    #[allow(dead_code)]
    pub async fn spawn() -> Self {
        Lazy::force(&TRACING);

        let address = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
        let server = lib_rpc::Server::new(address).await.expect("failed to bind test RPC server");

        let bound_address = server.address().expect("failed to get bound address");

        tokio::spawn(server.run());

        Self {
            address: bound_address,
        }
    }

    /// Connect a [`UtilitiesServiceClient`] to this test server.
    #[allow(dead_code)]
    pub async fn rpc_client(&self) -> lib_rpc::UtilitiesServiceClient<tonic::transport::Channel> {
        lib_rpc::UtilitiesServiceClient::connect(format!("http://{}", self.address))
            .await
            .expect("failed to connect to test RPC server")
    }
}
