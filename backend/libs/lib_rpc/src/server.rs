//-- ./backend/libs/lib_rpc/src/server.rs

//! gRPC server that hosts all RPC services.

use std::net::SocketAddr;

use tokio_stream::wrappers::TcpListenerStream;

use crate::services::{UtilitiesServiceImpl, UtilitiesServiceServer};

/// gRPC server bound to a TCP listener.
///
/// Construction and serving are split so that integration tests can bind to
/// port `0`, read back the assigned port via [`address`](Self::address),
/// and then spawn [`run`](Self::run) in a background task.
pub struct Server {
    listener: tokio::net::TcpListener,
}

impl Server {
    /// Bind the gRPC server to the given socket address.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Transport`] if the TCP listener cannot bind.
    pub async fn new(address: SocketAddr) -> crate::Result<Self> {
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        tracing::debug!("RPC server bound to {}", listener.local_addr().unwrap());

        Ok(Self { listener })
    }

    /// Returns the local address the server is bound to.
    pub fn address(&self) -> crate::Result<SocketAddr> {
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

        tracing::info!("RPC server listening on http://{addr}");

        tonic::transport::Server::builder()
            .add_service(utilities)
            .serve_with_incoming(incoming)
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(())
    }
}
