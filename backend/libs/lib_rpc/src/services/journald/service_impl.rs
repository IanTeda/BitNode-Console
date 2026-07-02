//! Concrete [`JournaldService`] implementation.

use crate::generated_protos::journald::journald_service_server::JournaldService;
use crate::generated_protos::journald::{
    GetLogsRequest, GetLogsResponse, LogEntry, StreamLogsRequest,
};

/// Concrete implementation of the [`JournaldService`] gRPC trait.
#[derive(Debug, Default)]
pub struct JournaldServiceImpl;

#[tonic::async_trait]
impl JournaldService for JournaldServiceImpl {
    type StreamLogsStream = super::stream_logs::LogStream;

    async fn get_logs(
        &self,
        request: tonic::Request<GetLogsRequest>,
    ) -> std::result::Result<tonic::Response<GetLogsResponse>, tonic::Status> {
        super::get_logs::handle(request).await
    }

    async fn stream_logs(
        &self,
        request: tonic::Request<StreamLogsRequest>,
    ) -> std::result::Result<tonic::Response<Self::StreamLogsStream>, tonic::Status> {
        super::stream_logs::handle(request).await
    }
}
