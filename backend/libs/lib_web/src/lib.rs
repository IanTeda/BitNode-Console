//-- ./backend/libs/lib_web/src/lib.rs

//! HTTP server library for the `BitNode` Console backend.

//--- Internal modules
mod error;
mod server;

//--- Re-export to flatten the module hierarchy
pub use error::Error;
pub use server::HttpServer;

/// Convenience [`Result`] alias for [`HttpError`].
pub type Result<T> = std::result::Result<T, Error>;
