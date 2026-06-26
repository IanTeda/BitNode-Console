//-- ./backend/libs/lib_auth/src/domains.rs

//! Auth domain types.
//!
//! This module provides domain types for authentication, including
//! password hashing and verification using Argon2id.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, SecretString};

/// A hashed password stored in PHC string format.
///
/// Wraps an Argon2id password hash. Construct from a plain-text password
/// via [`PasswordHash::from_password`], or from an existing PHC hash
/// string (e.g. loaded from settings) via [`TryFrom<String>`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Returns a configured Argon2id hasher instance.
    fn argon2() -> Argon2<'static> {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).expect("hardcoded Argon2 params are valid"),
        )
    }

    /// Hashes a plain-text password and returns a new [`PasswordHash`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PasswordHash`] if hashing fails.
    pub fn from_password(password: &SecretString) -> crate::Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Self::argon2()
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(|e| crate::Error::PasswordHash(e.to_string()))?;
        Ok(Self(hash.to_string()))
    }

    /// Verifies a plain-text password against this stored hash.
    ///
    /// Returns `true` if the password matches, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PasswordHash`] if the stored hash string
    /// is malformed and cannot be parsed.
    pub fn verify_password(&self, password: &SecretString) -> crate::Result<bool> {
        let parsed_hash = argon2::PasswordHash::new(self.as_ref())
            .map_err(|e| crate::Error::PasswordHash(e.to_string()))?;

        let verified = Argon2::default()
            .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
            .is_ok();

        Ok(verified)
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PasswordHash {
    type Error = crate::Error;

    /// Validates the string is a well-formed PHC hash before wrapping it.
    fn try_from(value: String) -> crate::Result<Self> {
        argon2::PasswordHash::new(&value)
            .map_err(|e| crate::Error::PasswordHash(e.to_string()))?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for PasswordHash {
    type Error = crate::Error;

    fn try_from(value: &str) -> crate::Result<Self> {
        Self::try_from(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_password() -> SecretString {
        SecretString::from("hunter2".to_string())
    }

    fn test_wrong_password() -> SecretString {
        SecretString::from("wrong_password".to_string())
    }

    #[test]
    fn from_password_produces_valid_phc_hash() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        assert!(
            hash.as_ref().starts_with("$argon2id$"),
            "expected PHC format hash, got: {}",
            hash.as_ref()
        );
    }

    #[test]
    fn verify_correct_password_returns_true() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        let result = hash.verify_password(&test_password())
            .expect("verification should not error");

        assert!(result);
    }

    #[test]
    fn verify_wrong_password_returns_false() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        let result = hash.verify_password(&test_wrong_password())
            .expect("verification should not error");

        assert!(!result);
    }

    #[test]
    fn from_password_produces_unique_hashes() {
        let hash_a = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");
        let hash_b = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        assert_ne!(hash_a, hash_b, "different salts should produce different hashes");
    }

    #[test]
    fn try_from_valid_hash_string_succeeds() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");
        let hash_string = hash.as_ref().to_string();

        let parsed = PasswordHash::try_from(hash_string)
            .expect("valid PHC string should parse");

        assert_eq!(parsed, hash);
    }

    #[test]
    fn try_from_invalid_string_fails() {
        let result = PasswordHash::try_from("not-a-valid-hash".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn try_from_empty_string_fails() {
        let result = PasswordHash::try_from(String::new());

        assert!(result.is_err());
    }

    #[test]
    fn try_from_str_ref_works() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");
        let hash_str = hash.as_ref();

        let parsed = PasswordHash::try_from(hash_str)
            .expect("valid PHC string should parse");

        assert_eq!(parsed, hash);
    }

    #[test]
    fn as_ref_returns_inner_hash() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        assert!(!hash.as_ref().is_empty());
        assert!(hash.as_ref().starts_with("$argon2id$"));
    }

    #[test]
    fn clone_produces_equal_value() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");
        let cloned = hash.clone();

        assert_eq!(hash, cloned);
    }

    #[test]
    fn debug_format_includes_struct_name() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        assert!(format!("{hash:?}").contains("PasswordHash"));
    }

    #[test]
    fn roundtrip_hash_then_parse_then_verify() {
        let hash = PasswordHash::from_password(&test_password())
            .expect("hashing should succeed");

        let parsed = PasswordHash::try_from(hash.as_ref().to_string())
            .expect("roundtrip parse should succeed");

        let result = parsed.verify_password(&test_password())
            .expect("verification should not error");

        assert!(result);
    }
}
