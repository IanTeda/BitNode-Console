//! Logout handler for the Authentication gRPC service.

use crate::services::authentication::{LogoutRequest, LogoutResponse};

/// Handle a logout request.
///
/// Token invalidation requires a server-side token store (to persist revoked JTIs)
/// which is not yet implemented.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<LogoutRequest>,
) -> std::result::Result<tonic::Response<LogoutResponse>, tonic::Status> {
    tracing::debug!("Logout request received from {:?}", request.remote_addr());
    tracing::info!("Token invalidation not yet implemented");
    Err(tonic::Status::unimplemented(
        "logout is not yet implemented",
    ))
}

#[cfg(test)]
mod tests {
    use super::{LogoutRequest, LogoutResponse, handle};

    fn logout_request() -> tonic::Request<LogoutRequest> {
        tonic::Request::new(LogoutRequest {})
    }

    #[tokio::test]
    async fn logout_returns_unimplemented() {
        let status = handle(logout_request()).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unimplemented);
    }

    #[tokio::test]
    async fn logout_unimplemented_error_message() {
        let status = handle(logout_request()).await.unwrap_err();
        assert_eq!(status.message(), "logout is not yet implemented");
    }

    // --- message construction ---

    #[test]
    fn logout_request_default_is_empty() {
        assert_eq!(LogoutRequest::default(), LogoutRequest {});
    }

    #[test]
    fn logout_response_default_is_empty() {
        assert_eq!(LogoutResponse::default(), LogoutResponse {});
    }
}
