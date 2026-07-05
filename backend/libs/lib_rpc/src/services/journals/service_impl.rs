//! Concrete [`JournalsService`] implementation.

use crate::services::journals::JournalsService;
use crate::services::journals::{
    GetJournalsRequest, GetJournalsResponse, JournalsEntry, StreamJournalsRequest,
};

/// Concrete implementation of the [`JournalsService`] gRPC trait.
#[derive(Debug)]
pub struct JournalsServiceImpl {
    unit_name: String,
}

impl JournalsServiceImpl {
    /// Create a new service instance scoped to the given systemd unit.
    pub fn new(unit_name: impl Into<String>) -> Self {
        Self {
            unit_name: unit_name.into(),
        }
    }
}

#[tonic::async_trait]
impl JournalsService for JournalsServiceImpl {
    type StreamJournalsStream = super::stream_journals::JournalStream;

    async fn get_journals(
        &self,
        request: tonic::Request<GetJournalsRequest>,
    ) -> std::result::Result<tonic::Response<GetJournalsResponse>, tonic::Status> {
        super::get_journals::handle(&self.unit_name, request)
            .await
            .map_err(Into::into)
    }

    async fn stream_journals(
        &self,
        request: tonic::Request<StreamJournalsRequest>,
    ) -> std::result::Result<tonic::Response<Self::StreamJournalsStream>, tonic::Status> {
        super::stream_journals::handle(request)
            .await
            .map_err(Into::into)
    }
}
