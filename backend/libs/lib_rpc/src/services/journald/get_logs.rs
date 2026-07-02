//! GetLogs handler for the Journald gRPC service.

use super::{GetLogsRequest, GetLogsResponse};

/// Handle a `GetLogs` request — return a page of journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    request: tonic::Request<GetLogsRequest>,
) -> std::result::Result<tonic::Response<GetLogsResponse>, tonic::Status> {
    tracing::debug!("GetLogs request from {:?}", request.remote_addr());

    Err(tonic::Status::unimplemented(
        "GetLogs is not yet implemented",
    ))
}
