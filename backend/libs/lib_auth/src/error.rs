//-- ./backend/libs/lib_auth/src/error.rs

//! Auth Library Crate Error
//!
//! This module provides error types for the auth module.

/// Errors that can occur in the auth library.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Catch-all for errors that do not fit a more specific variant.
    #[error("Auth error: {0}")]
    Generic(String),

    /// Error encountered while hashing or verifying a password.
    #[error("Password hash error: {0}")]
    PasswordHash(String),

    /// Error encoding or decoding a JSON Web Token.
    #[error("Token error: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),

    /// A token of the wrong type was supplied (e.g. refresh token used as access token).
    #[error("invalid token type: expected {expected}, got {got}")]
    InvalidTokenType { expected: String, got: String },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_error_displays_message() {
        let error = Error::Generic("something went wrong".to_string());

        assert_eq!(error.to_string(), "Auth error: something went wrong");
    }

    #[test]
    fn password_hash_error_displays_message() {
        let error = Error::PasswordHash("invalid hash".to_string());

        assert_eq!(error.to_string(), "Password hash error: invalid hash");
    }

    #[test]
    fn debug_format_includes_variant_name() {
        assert!(format!("{:?}", Error::Generic("x".to_string())).contains("Generic"));
        assert!(format!("{:?}", Error::PasswordHash("x".to_string())).contains("PasswordHash"));
    }

    #[test]
    fn result_alias_works_with_ok() {
        let result: crate::Result<u32> = Ok(42);

        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn result_alias_works_with_err() {
        let result: crate::Result<u32> = Err(Error::Generic("fail".to_string()));

        assert!(result.is_err());
    }
}
