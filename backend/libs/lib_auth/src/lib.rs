//-- ./backend/libs/lib_auth/src/lib.rs

//! Auth Library Module
//!
//! This library provides authentication functionality.

mod domains;
mod error;

/// Re-export Auth domain types.
pub use domains::{AccessToken, PasswordHash, RefreshToken, TokenClaim, TokenType};

/// Re-export Auth error type.
pub use error::Error;

/// Result type alias used across the auth module.
pub type Result<T> = std::result::Result<T, Error>;
