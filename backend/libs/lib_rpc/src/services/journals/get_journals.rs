//! `GetJournals` handler for the Journals gRPC service.
//!
//! Returns a single page of journal log entries for the configured systemd
//! unit.  The seek runs synchronously on the calling Tokio task — for very
//! large result sets this can block the worker; page sizes should be kept
//! reasonable by the client.
//!
//! Journal errors are propagated via `?` and converted to
//! [`tonic::Status::internal`] by the [`crate::Error`] `→` [`tonic::Status`]
//! impl.

use crate::services::journals::{GetJournalsRequest, GetJournalsResponse};

/// Handle a `GetJournals` RPC — seek the journal and return one page of entries.
///
/// `unit_name` comes from the service configuration (e.g. `"bitcoind.service"`);
/// it is not part of the client request.  It is injected into the domain
/// [`lib_journals::Query`] after conversion so that the conversion layer stays
/// ignorant of server-side configuration.
///
/// # Errors
///
/// Returns `Err` (which tonic converts to [`tonic::Status::internal`]) if:
/// - the journal cannot be opened ([`lib_journals::Connection::open_current_user`]), or
/// - the seek fails (I/O error on the underlying journal handle).
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    unit_name: &str,
    request: tonic::Request<GetJournalsRequest>,
) -> crate::Result<tonic::Response<GetJournalsResponse>> {
    tracing::debug!("GetJournals request from {:?}", request.remote_addr());

    let mut journal_query: lib_journals::Query = request.into_inner().into();
    journal_query.unit_name = unit_name;

    let mut conn = lib_journals::Connection::open_current_user()?;

    let entries = journal_query.seek(&mut conn)?;

    let response: GetJournalsResponse = entries.into();

    Ok(tonic::Response::new(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_protos::journals::GetJournalsRequest;

    fn empty_request() -> tonic::Request<GetJournalsRequest> {
        tonic::Request::new(GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us: None,
            priority: None,
            pagination: None,
        })
    }

    /// The handler must succeed for a well-formed request, even when the
    /// journal is empty for the given unit.
    #[tokio::test]
    async fn handle_returns_ok() {
        let result = handle("bitcoind.service", empty_request()).await;
        assert!(result.is_ok());
    }

    /// A unit with no matching journal entries yields an empty `entries` vec,
    /// not an error.
    #[tokio::test]
    async fn nonexistent_unit_returns_empty_entries() {
        let response = handle("__nonexistent_unit__.service", empty_request())
            .await
            .unwrap()
            .into_inner();
        assert!(response.entries.is_empty());
    }

    /// `GetJournalsResponse` always carries a `pagination` field — the
    /// `Page → GetJournalsResponse` conversion always sets it to `Some`.
    #[tokio::test]
    async fn response_always_contains_pagination_metadata() {
        let response = handle("__nonexistent_unit__.service", empty_request())
            .await
            .unwrap()
            .into_inner();
        assert!(response.pagination.is_some());
    }
}
