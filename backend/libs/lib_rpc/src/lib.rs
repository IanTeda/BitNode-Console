mod error;
mod generated_protos;
mod server;
pub(crate) mod services;

/// Convenience [`Result`] alias using [`Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;

//--- Re-export to flatten module hierarchy
pub use error::Error;
pub use generated_protos::utilities_service_client::UtilitiesServiceClient;
pub use generated_protos::utilities_service_server::{UtilitiesService, UtilitiesServiceServer};
pub use generated_protos::{PingRequest, PingResponse};
pub use server::Server;
pub use services::UtilitiesServiceImpl;
