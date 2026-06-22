mod error;
mod generated_protos;
mod utilities;

pub use error::Error;
pub use utilities::*;

/// Convenience [`Result`] alias using [`RpcError`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;
