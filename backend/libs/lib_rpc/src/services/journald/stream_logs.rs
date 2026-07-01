//! StreamLogs handler for the Journald gRPC service.

use std::pin::Pin;

use tokio_stream::Stream;

use super::{LogEntry, StreamLogsRequest};

/// Pinned boxed stream of log entries yielded by [`handle`].
pub(super) type LogStream =
    Pin<Box<dyn Stream<Item = std::result::Result<LogEntry, tonic::Status>> + Send>>;

/// Handle a `StreamLogs` request — stream live journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<StreamLogsRequest>,
) -> std::result::Result<tonic::Response<LogStream>, tonic::Status> {
    tracing::debug!("StreamLogs request from {:?}", request.remote_addr());

    Err(tonic::Status::unimplemented("StreamLogs is not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::{StreamLogsRequest, handle};

    fn stream_logs_request() -> tonic::Request<StreamLogsRequest> {
        tonic::Request::new(StreamLogsRequest { tail_lines: 0 })
    }

    #[tokio::test]
    async fn stream_logs_returns_unimplemented() {
        match handle(stream_logs_request()).await {
            Err(err) => assert_eq!(err.code(), tonic::Code::Unimplemented),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }

    #[tokio::test]
    async fn stream_logs_unimplemented_message_is_not_empty() {
        match handle(stream_logs_request()).await {
            Err(err) => assert!(!err.message().is_empty()),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }

    #[tokio::test]
    async fn stream_logs_with_nonzero_tail_returns_unimplemented() {
        let request = tonic::Request::new(StreamLogsRequest { tail_lines: 100 });
        match handle(request).await {
            Err(err) => assert_eq!(err.code(), tonic::Code::Unimplemented),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }
}
