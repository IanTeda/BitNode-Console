//! GetLogs handler for the Journald gRPC service.

use super::{GetLogsRequest, GetLogsResponse};

/// Handle a `GetLogs` request — return a page of journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<GetLogsRequest>,
) -> std::result::Result<tonic::Response<GetLogsResponse>, tonic::Status> {
    tracing::debug!("GetLogs request from {:?}", request.remote_addr());

    Err(tonic::Status::unimplemented("GetLogs is not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::{GetLogsRequest, handle};

    fn get_logs_request() -> tonic::Request<GetLogsRequest> {
        tonic::Request::new(GetLogsRequest { pagination: None })
    }

    #[tokio::test]
    async fn get_logs_returns_unimplemented() {
        let err = handle(get_logs_request()).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unimplemented);
    }

    #[tokio::test]
    async fn get_logs_unimplemented_message_is_not_empty() {
        let err = handle(get_logs_request()).await.unwrap_err();
        assert!(!err.message().is_empty());
    }
}
