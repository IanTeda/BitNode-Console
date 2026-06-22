//! Implementation of the gRPC Utilities service.

pub use crate::generated_protos::utilities_service_server::{
    UtilitiesService, UtilitiesServiceServer,
};

pub use crate::generated_protos::{PingRequest, PingResponse};

/// Concrete implementation of the [`UtilitiesService`] gRPC trait.
#[derive(Debug, Default)]
pub struct UtilitiesServiceImpl;

#[tonic::async_trait]
impl UtilitiesService for UtilitiesServiceImpl {
    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> std::result::Result<tonic::Response<PingResponse>, tonic::Status> {
        tracing::info!("Ping request from {:?}", request.remote_addr());

        let reply = PingResponse {
            pong: "Pong...".to_string(),
        };

        Ok(tonic::Response::new(reply))
    }
}
