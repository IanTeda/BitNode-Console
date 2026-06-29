//! Ping handler for the Utilities gRPC service.

use super::{PingRequest, PingResponse};

/// Handle a ping request — return a static pong response.
pub(super) async fn handle(
    request: tonic::Request<PingRequest>,
) -> std::result::Result<tonic::Response<PingResponse>, tonic::Status> {
    tracing::info!("Ping request from {:?}", request.remote_addr());

    let reply = PingResponse {
        pong: "Pong...".to_string(),
    };

    Ok(tonic::Response::new(reply))
}

#[cfg(test)]
mod tests {
    use super::{PingRequest, PingResponse, handle};

    fn ping_request() -> tonic::Request<PingRequest> {
        tonic::Request::new(PingRequest {})
    }

    #[tokio::test]
    async fn ping_returns_ok() {
        assert!(handle(ping_request()).await.is_ok());
    }

    #[tokio::test]
    async fn ping_response_contains_pong() {
        let response = handle(ping_request()).await.unwrap();
        assert_eq!(response.into_inner().pong, "Pong...");
    }

    #[tokio::test]
    async fn ping_is_idempotent() {
        let first = handle(ping_request()).await.unwrap().into_inner();
        let second = handle(ping_request()).await.unwrap().into_inner();
        assert_eq!(first.pong, second.pong);
    }

    #[tokio::test]
    async fn ping_response_pong_is_not_empty() {
        let response = handle(ping_request()).await.unwrap();
        assert!(!response.into_inner().pong.is_empty());
    }

    #[test]
    fn ping_request_default_is_empty() {
        assert_eq!(PingRequest::default(), PingRequest {});
    }

    #[test]
    fn ping_response_can_be_constructed_with_custom_message() {
        let response = PingResponse { pong: "custom".to_string() };
        assert_eq!(response.pong, "custom");
    }
}
