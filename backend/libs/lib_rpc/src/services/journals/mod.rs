//! gRPC Journals service — delegates each RPC to its own handler module.

pub(crate) mod follow_journals;
pub(crate) mod get_journals;
mod journal_follow_from;
mod journal_page_from;
mod journal_query_from;
pub(crate) mod service_impl;

pub use crate::generated_protos::journals::journals_service_client::JournalsServiceClient;
pub use crate::generated_protos::journals::journals_service_server::{
    JournalsService, JournalsServiceServer,
};

// --- Get Journals
pub use crate::generated_protos::journals::{
    GetJournalsRequest, GetJournalsResponse, JournalsEntry,
};

// --- Follow Journals
pub use crate::generated_protos::journals::FollowJournalsRequest;

pub use service_impl::JournalsServiceImpl;
