//! Concrete [`JournalsService`] implementation.

use crate::services::journals::JournalsService;
use crate::services::journals::{
    GetJournalsRequest, GetJournalsResponse, JournalsEntry, StreamJournalsRequest,
};

/// Concrete implementation of the [`JournalsService`] gRPC trait.
#[derive(Debug, Default)]
pub struct JournalsServiceImpl;

#[tonic::async_trait]
impl JournalsService for JournalsServiceImpl {
    type StreamJournalsStream = super::stream_journals::JournalStream;

    async fn get_journals(
        &self,
        request: tonic::Request<GetJournalsRequest>,
    ) -> std::result::Result<tonic::Response<GetJournalsResponse>, tonic::Status> {
        super::get_journals::handle(request).await
    }

    async fn stream_journals(
        &self,
        request: tonic::Request<StreamJournalsRequest>,
    ) -> std::result::Result<tonic::Response<Self::StreamJournalsStream>, tonic::Status> {
        super::stream_journals::handle(request).await
    }
}
