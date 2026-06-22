mod error;
mod generated_protos;
pub(crate) mod services;
mod server;

/// Convenience [`Result`] alias using [`Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;

//--- Re-export to flatten module hierarchy
pub use error::Error;
pub use server::Server;
pub use services::UtilitiesServiceImpl;
pub use generated_protos::utilities_service_server::{UtilitiesService, UtilitiesServiceServer};
pub use generated_protos::{PingRequest, PingResponse};
pub use generated_protos::utilities_service_client::UtilitiesServiceClient;
