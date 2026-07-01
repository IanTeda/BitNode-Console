//! gRPC Journald service — delegates each RPC to its own handler module.

pub(super) mod get_logs;
pub(super) mod service_impl;
pub(super) mod stream_logs;

pub use crate::generated_protos::journald::journald_service_server::{
    JournaldService, JournaldServiceServer,
};
pub use crate::generated_protos::journald::{
    GetLogsRequest, GetLogsResponse, LogEntry, StreamLogsRequest,
};
pub use service_impl::JournaldServiceImpl;
