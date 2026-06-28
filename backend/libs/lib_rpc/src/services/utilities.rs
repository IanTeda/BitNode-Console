//! Implementation of the gRPC Utilities service.

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
        tracing::info!("Ping request from {:?}", request.remote_addr());

        let reply = PingResponse {
            pong: "Pong...".to_string(),
        };

        Ok(tonic::Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ping_request() -> tonic::Request<PingRequest> {
        tonic::Request::new(PingRequest {})
    }

    #[tokio::test]
    async fn ping_returns_ok() {
        let service = UtilitiesServiceImpl;
        let response = service.ping(ping_request()).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn ping_response_contains_pong() {
        let service = UtilitiesServiceImpl;
        let response = service.ping(ping_request()).await.unwrap();

        assert_eq!(response.into_inner().pong, "Pong...");
    }

    #[tokio::test]
    async fn ping_is_idempotent() {
        let service = UtilitiesServiceImpl;

        let first = service.ping(ping_request()).await.unwrap().into_inner();
        let second = service.ping(ping_request()).await.unwrap().into_inner();

        assert_eq!(first.pong, second.pong);
    }

    #[test]
    fn service_impl_has_debug() {
        let service = UtilitiesServiceImpl;
        let debug = format!("{service:?}");

        assert!(debug.contains("UtilitiesServiceImpl"));
    }

    #[test]
    fn service_impl_default_creates_instance() {
        let _service = UtilitiesServiceImpl::default();
    }

    #[tokio::test]
    async fn ping_response_pong_is_not_empty() {
        let service = UtilitiesServiceImpl;
        let response = service.ping(ping_request()).await.unwrap();

        assert!(!response.into_inner().pong.is_empty());
    }

    #[test]
    fn ping_request_default_is_empty() {
        let request = PingRequest::default();
        assert_eq!(request, PingRequest {});
    }

    #[test]
    fn ping_response_can_be_constructed_with_custom_message() {
        let response = PingResponse {
            pong: "custom".to_string(),
        };

        assert_eq!(response.pong, "custom");
    }

    #[test]
    fn utilities_service_server_wraps_impl() {
        let service = UtilitiesServiceImpl::default();
        let _server = UtilitiesServiceServer::new(service);
    }
}
