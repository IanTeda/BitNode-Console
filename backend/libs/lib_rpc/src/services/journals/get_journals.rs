//! GetJournals handler for the Journals gRPC service.

use crate::services::journals::{GetJournalsRequest, GetJournalsResponse};

/// Handle a `GetJournals` request — return a page of journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    unit_name: &str,
    request: tonic::Request<GetJournalsRequest>,
) -> crate::Result<tonic::Response<GetJournalsResponse>> {
    tracing::debug!("GetJournals request from {:?}", request.remote_addr());

    //--- Convert the request into a `JournalQuery` and set the unit name
    let mut journal_query: lib_journals::JournalQuery = request.into_inner().into();
    journal_query.unit_name = unit_name;

    //--- Open a journal connection and seek the entries
    let mut conn = lib_journals::JournalConnection::open_current_user()?;

    let entries = journal_query.seek(&mut conn)?;

    //--- Convert the entries into a `GetJournalsResponse` and return it
    let response: GetJournalsResponse = entries.into();

    Ok(tonic::Response::new(response))
}
