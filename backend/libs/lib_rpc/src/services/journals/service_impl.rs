//! Concrete [`JournalsServiceImpl`] that wires the tonic [`JournalsService`]
//! trait to the per-RPC handler functions.
//!
//! `unit_name` is the only piece of server-side configuration held here; it is
//! injected into every handler call so that handler modules remain unaware of
//! how the service is configured.  Each method converts a [`crate::Error`]
//! returned by the handler into a [`tonic::Status`] via [`Into::into`].

use crate::services::journals::JournalsService;
use crate::services::journals::{
    FollowJournalsRequest, GetJournalsRequest, GetJournalsResponse,
};

/// Concrete implementation of the [`JournalsService`] gRPC trait.
///
/// Holds the server-side configuration (currently just the systemd unit name)
/// that is threaded through to every handler call.
#[derive(Debug)]
pub struct JournalsServiceImpl {
    /// Fully-qualified systemd unit name whose journal this service exposes,
    /// e.g. `"bitcoind.service"`.  Sourced from server configuration — not
    /// supplied by clients.
    unit_name: String,
}

impl JournalsServiceImpl {
    /// Create a new service instance scoped to the given systemd unit.
    ///
    /// `unit_name` should be a fully-qualified systemd unit name such as
    /// `"bitcoind.service"`.  It is stored and passed to every handler call
    /// so handlers can filter the journal without knowing how the service is
    /// configured.
    pub fn new(unit_name: impl Into<String>) -> Self {
        Self {
            unit_name: unit_name.into(),
        }
    }
}

#[tonic::async_trait]
impl JournalsService for JournalsServiceImpl {
    /// Pinned streaming response type returned by [`Self::follow_journals`].
    type FollowJournalsStream = super::follow_journals::JournalStream;

    /// Return a single page of journal log entries.
    ///
    /// Delegates to [`super::get_journals::handle`], converting any
    /// [`crate::Error`] to [`tonic::Status`] via [`Into::into`].
    async fn get_journals(
        &self,
        request: tonic::Request<GetJournalsRequest>,
    ) -> std::result::Result<tonic::Response<GetJournalsResponse>, tonic::Status> {
        super::get_journals::handle(&self.unit_name, request)
            .await
            .map_err(Into::into)
    }

    /// Stream live journal log entries until the client disconnects.
    ///
    /// Delegates to [`super::follow_journals::handle`], which spawns a
    /// dedicated OS thread for the blocking `journal.wait(None)` call and
    /// bridges entries back via an `mpsc` channel.  Any [`crate::Error`] is
    /// converted to [`tonic::Status`] via [`Into::into`].
    async fn follow_journals(
        &self,
        request: tonic::Request<FollowJournalsRequest>,
    ) -> std::result::Result<tonic::Response<Self::FollowJournalsStream>, tonic::Status> {
        super::follow_journals::handle(&self.unit_name, request)
            .await
            .map_err(Into::into)
    }
}
