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

#[cfg(test)]
mod tests {
    use crate::generated_protos::journald::journald_service_server::{
        JournaldService, JournaldServiceServer,
    };
    use crate::generated_protos::journald::{GetLogsRequest, StreamLogsRequest};

    use super::JournaldServiceImpl;

    #[test]
    fn service_impl_has_debug() {
        let service = JournaldServiceImpl;
        assert!(format!("{service:?}").contains("JournaldServiceImpl"));
    }

    #[test]
    fn service_impl_default_creates_instance() {
        let _service = JournaldServiceImpl::default();
    }

    #[test]
    fn journald_service_server_wraps_impl() {
        let _server = JournaldServiceServer::new(JournaldServiceImpl::default());
    }

    #[tokio::test]
    async fn get_logs_returns_unimplemented() {
        let service = JournaldServiceImpl;
        let request = tonic::Request::new(GetLogsRequest { pagination: None });
        let err = service.get_logs(request).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unimplemented);
    }

    #[tokio::test]
    async fn stream_logs_returns_unimplemented() {
        let service = JournaldServiceImpl;
        let request = tonic::Request::new(StreamLogsRequest { tail_lines: 0 });
        match service.stream_logs(request).await {
            Err(err) => assert_eq!(err.code(), tonic::Code::Unimplemented),
            Ok(_) => panic!("expected Unimplemented error"),
        }
    }
}
