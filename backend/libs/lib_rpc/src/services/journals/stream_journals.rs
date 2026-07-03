//! StreamJournals handler for the Journals gRPC service.

use std::pin::Pin;

use tokio_stream::Stream;

use super::{JournalsEntry, StreamJournalsRequest};

/// Pinned boxed stream of journal entries yielded by [`handle`].
pub(super) type JournalStream =
    Pin<Box<dyn Stream<Item = std::result::Result<JournalsEntry, tonic::Status>> + Send>>;

/// Handle a `StreamJournals` request — stream live journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<StreamJournalsRequest>,
) -> std::result::Result<tonic::Response<JournalStream>, tonic::Status> {
    tracing::debug!("StreamJournals request from {:?}", request.remote_addr());

    Err(tonic::Status::unimplemented(
        "StreamJournals is not yet implemented",
    ))
}

#[cfg(test)]
mod tests {
    use super::{StreamJournalsRequest, handle};

    fn stream_journals_request() -> tonic::Request<StreamJournalsRequest> {
        tonic::Request::new(StreamJournalsRequest { tail_lines: 0 })
    }

    #[tokio::test]
    async fn stream_journals_returns_unimplemented() {
        match handle(stream_journals_request()).await {
            Err(err) => assert_eq!(err.code(), tonic::Code::Unimplemented),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }

    #[tokio::test]
    async fn stream_journals_unimplemented_message_is_not_empty() {
        match handle(stream_journals_request()).await {
            Err(err) => assert!(!err.message().is_empty()),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }

    #[tokio::test]
    async fn stream_journals_with_nonzero_tail_returns_unimplemented() {
        let request = tonic::Request::new(StreamJournalsRequest { tail_lines: 100 });
        match handle(request).await {
            Err(err) => assert_eq!(err.code(), tonic::Code::Unimplemented),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }
}
