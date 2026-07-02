mod error;
mod generated_protos;
pub(crate) mod interceptors;
mod pagination_from;
mod server;
pub mod services;

// Re-expose common types at the crate root so prost-generated cross-package
// references (super::super::common::v1::*) resolve correctly. Prost navigates
// two levels up from generated_protos::journals to reach the crate root, then
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

pub use server::Server;
