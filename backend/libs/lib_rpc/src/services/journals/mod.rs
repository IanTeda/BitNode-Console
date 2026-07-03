//! gRPC Journals service — delegates each RPC to its own handler module.

pub(super) mod get_journals;
pub(super) mod service_impl;
pub(super) mod stream_journals;

pub use crate::generated_protos::journals::journals_service_server::{
    JournalsService, JournalsServiceServer,
};
pub use crate::generated_protos::journals::{
    GetJournalsRequest, GetJournalsResponse, JournalsEntry, StreamJournalsRequest,
};

pub use service_impl::JournalsServiceImpl;
