//! Server Crate Error

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    // Start with generic error during development and then expand error types below as needed.
    #[error("Generic error {0}")]
    GenericError(String),
}
