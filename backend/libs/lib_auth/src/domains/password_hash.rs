//! Argon2id password hashing and verification.

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, SecretString};

/// A hashed password stored in PHC string format.
///
/// Wraps an Argon2id password hash. Construct from a plain-text password
/// via [`PasswordHash::from_password`], or wrap an existing PHC hash string
/// (e.g. loaded from settings) via [`PasswordHash::from_hash`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Returns a configured Argon2id hasher.
    ///
    /// Params: 15 MiB memory, 2 iterations, 1 lane — suitable for a
    /// low-parallelism server. Increase memory cost before production use.
    fn argon2() -> Argon2<'static> {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15_000, 2, 1, None).expect("hardcoded Argon2 params are valid"),
        )
    }

    /// Hash a plain-text password and return a new [`PasswordHash`].
    ///
    /// Each call generates a fresh random salt, so two hashes of the same
    /// password will differ. Use [`verify_password`] to check them.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PasswordHash`] if hashing fails.
    ///
    /// [`verify_password`]: Self::verify_password
    #[tracing::instrument(skip(password))]
    pub fn from_password(password: &SecretString) -> crate::Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Self::argon2()
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(|e| crate::Error::PasswordHash(e.to_string()))?;
        tracing::debug!("password hashed");
        Ok(Self(hash.to_string()))
    }

    /// Wrap an existing Argon2id PHC hash string in a [`PasswordHash`].
    ///
    /// Validates that `hash` is a well-formed PHC string before wrapping it.
    /// Use this when loading a pre-computed hash from configuration or storage.
    /// To hash a plain-text password instead, use [`PasswordHash::from_password`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PasswordHash`] if `hash` is not a valid PHC string.
    #[tracing::instrument()]
    pub fn from_hash(hash: &str) -> crate::Result<Self> {
        Self::try_from(hash)
    }

    /// Verify a plain-text password against this stored hash.
    ///
    /// Returns `true` if the password matches, `false` otherwise.
    /// The comparison is performed in constant time by the argon2 library.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PasswordHash`] only if the stored PHC string
    /// is malformed and cannot be parsed — not on a wrong password.
    #[tracing::instrument(skip(self, password))]
    pub fn verify_password(&self, password: &SecretString) -> crate::Result<bool> {
        // `verify_password` reads algorithm and params from the PHC string
        // itself, so the Argon2 instance's own params are not used here.
        let parsed_hash = argon2::PasswordHash::new(self.as_ref())
            .map_err(|e| crate::Error::PasswordHash(e.to_string()))?;

        let verified = Argon2::default()
            .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
            .is_ok();

        tracing::debug!(verified, "password verification complete");
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

    /// Validate that `value` is a well-formed PHC string before wrapping it.
    fn try_from(value: String) -> crate::Result<Self> {
        argon2::PasswordHash::new(&value).map_err(|e| crate::Error::PasswordHash(e.to_string()))?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for PasswordHash {
    type Error = crate::Error;

    /// Validate that `value` is a well-formed PHC string before wrapping it.
    fn try_from(value: &str) -> crate::Result<Self> {
        Self::try_from(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password() -> SecretString {
        SecretString::from("hunter2")
    }

    fn wrong_password() -> SecretString {
        SecretString::from("wrong_password")
    }

    // --- from_password ---

    #[test]
    fn from_password_produces_argon2id_phc_hash() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(hash.as_ref().starts_with("$argon2id$"));
    }

    #[test]
    fn from_password_produces_unique_hashes() {
        let a = PasswordHash::from_password(&password()).unwrap();
        let b = PasswordHash::from_password(&password()).unwrap();
        assert_ne!(a, b, "distinct salts must produce distinct hashes");
    }

    #[test]
    fn empty_password_can_be_hashed() {
        let empty = SecretString::from("");
        let hash = PasswordHash::from_password(&empty);
        assert!(hash.is_ok());
    }

    #[test]
    fn unicode_password_can_be_hashed() {
        let unicode = SecretString::from("correct-horse-🔋-staple");
        let hash = PasswordHash::from_password(&unicode);
        assert!(hash.is_ok());
    }

    // --- verify_password ---

    #[test]
    fn correct_password_verifies() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(hash.verify_password(&password()).unwrap());
    }

    #[test]
    fn wrong_password_does_not_verify() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(!hash.verify_password(&wrong_password()).unwrap());
    }

    #[test]
    fn empty_password_verifies_against_its_own_hash() {
        let empty = SecretString::from("");
        let hash = PasswordHash::from_password(&empty).unwrap();
        assert!(hash.verify_password(&empty).unwrap());
    }

    #[test]
    fn empty_password_does_not_verify_non_empty() {
        let empty = SecretString::from("");
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(!hash.verify_password(&empty).unwrap());
    }

    #[test]
    fn unicode_password_verifies() {
        let unicode = SecretString::from("correct-horse-🔋-staple");
        let hash = PasswordHash::from_password(&unicode).unwrap();
        assert!(hash.verify_password(&unicode).unwrap());
    }

    #[test]
    fn password_verification_is_case_sensitive() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        let upper = SecretString::from("Hunter2");
        assert!(!hash.verify_password(&upper).unwrap());
    }

    #[test]
    fn malformed_hash_returns_error_not_false() {
        let bad = PasswordHash(String::from("not-a-phc-string"));
        assert!(bad.verify_password(&password()).is_err());
    }

    // --- TryFrom ---

    #[test]
    fn try_from_valid_phc_string_succeeds() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        let parsed = PasswordHash::try_from(hash.as_ref().to_string()).unwrap();
        assert_eq!(parsed, hash);
    }

    #[test]
    fn try_from_valid_phc_str_ref_succeeds() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        let parsed = PasswordHash::try_from(hash.as_ref()).unwrap();
        assert_eq!(parsed, hash);
    }

    #[test]
    fn try_from_invalid_string_fails() {
        assert!(PasswordHash::try_from("not-a-valid-hash".to_string()).is_err());
    }

    #[test]
    fn try_from_empty_string_fails() {
        assert!(PasswordHash::try_from(String::new()).is_err());
    }

    #[test]
    fn roundtrip_hash_parse_verify() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        let parsed = PasswordHash::try_from(hash.as_ref().to_string()).unwrap();
        assert!(parsed.verify_password(&password()).unwrap());
    }

    // --- trait impls ---

    #[test]
    fn as_ref_returns_phc_string() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(hash.as_ref().starts_with("$argon2id$"));
    }

    #[test]
    fn clone_produces_equal_value() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert_eq!(hash.clone(), hash);
    }

    #[test]
    fn debug_format_includes_struct_name() {
        let hash = PasswordHash::from_password(&password()).unwrap();
        assert!(format!("{hash:?}").contains("PasswordHash"));
    }
}
