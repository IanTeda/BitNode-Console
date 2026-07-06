//! gRPC Journals service — delegates each RPC to its own handler module.

pub(crate) mod get_journals;
mod journal_page_from;
mod journal_query_from;
pub(crate) mod service_impl;
pub(crate) mod follow_journals;

pub use crate::generated_protos::journals::journals_service_client::JournalsServiceClient;
pub use crate::generated_protos::journals::journals_service_server::{
    JournalsService, JournalsServiceServer,
};

pub use crate::generated_protos::journals::{
    GetJournalsRequest, GetJournalsResponse, JournalsEntry, FollowJournalsRequest,
};

pub use service_impl::JournalsServiceImpl;
