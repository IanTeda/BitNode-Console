//! gRPC Utilities service — delegates each RPC to its own handler module.

mod ping;

pub use crate::generated_protos::utilities::utilities_service_server::{
    UtilitiesService, UtilitiesServiceServer,
};
pub use crate::generated_protos::utilities::{PingRequest, PingResponse};

/// Concrete implementation of the [`UtilitiesService`] gRPC trait.
#[derive(Debug, Default)]
pub struct UtilitiesServiceImpl;

#[tonic::async_trait]
impl UtilitiesService for UtilitiesServiceImpl {
    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> std::result::Result<tonic::Response<PingResponse>, tonic::Status> {
        ping::handle(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::{UtilitiesServiceImpl, UtilitiesServiceServer};

    #[test]
    fn service_impl_has_debug() {
        let service = UtilitiesServiceImpl;
        assert!(format!("{service:?}").contains("UtilitiesServiceImpl"));
    }

    #[test]
    fn service_impl_default_creates_instance() {
        let _service = UtilitiesServiceImpl::default();
    }

    #[test]
    fn utilities_service_server_wraps_impl() {
        let _server = UtilitiesServiceServer::new(UtilitiesServiceImpl::default());
    }
}
