//! GetJournals handler for the Journals gRPC service.

use super::{GetJournalsRequest, GetJournalsResponse};

/// Handle a `GetJournals` request — return a page of journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<GetJournalsRequest>,
) -> std::result::Result<tonic::Response<GetJournalsResponse>, tonic::Status> {
    tracing::debug!("GetJournals request from {:?}", request.remote_addr());

    Err(tonic::Status::unimplemented(
        "GetJournals is not yet implemented",
    ))
}
