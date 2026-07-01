mod error;
mod generated_protos;
mod interceptors;
mod server;
pub(crate) mod services;

// Re-expose common types at the crate root so prost-generated cross-package
// references (super::super::common::v1::*) resolve correctly. Prost navigates
// two levels up from generated_protos::journald to reach the crate root, then
// expects common::v1 to exist there.
pub(crate) mod common {
    pub mod v1 {
        pub use crate::generated_protos::common::*;
    }
}

/// Convenience [`Result`] alias using [`Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;

//--- Re-export to flatten module hierarchy
pub use error::Error;
pub use generated_protos::utilities::utilities_service_client::UtilitiesServiceClient;
pub use generated_protos::utilities::utilities_service_server::{
    UtilitiesService, UtilitiesServiceServer,
};
pub use generated_protos::utilities::{PingRequest, PingResponse};
pub use interceptors::AccessTokenInterceptor;
pub use interceptors::AllowedIpsInterceptor;
pub use server::Server;
pub use services::UtilitiesServiceImpl;
