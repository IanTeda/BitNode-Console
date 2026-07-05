//! Cursor-based pagination domain types.
//!
//! These types are transport-agnostic. Crates that speak proto (e.g. `lib_rpc`)
//! hold the `From` impls that convert to/from the generated proto messages.

pub mod direction;
pub mod request;
pub mod response;

pub use direction::Direction;
pub use request::{PaginationRequest, DEFAULT_PAGE_SIZE};
pub use response::PaginationResponse;
