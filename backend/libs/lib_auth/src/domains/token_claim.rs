//! JWT claim payload for access and refresh tokens.
//!
//! # References
//!
//! * [Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken)
//! * [JSON Web Token (JWT)](https://www.rfc-editor.org/rfc/rfc7519)
//! * [IANA JWT registry](https://www.iana.org/assignments/jwt/jwt.xhtml)

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::{ExposeSecret, SecretString};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use super::TokenType;

/// JWT issuer identifier.
pub static TOKEN_ISSUER: &str = "BitNode-Console";

/// JWT audience for access tokens (RPC API).
static ACCESS_AUDIENCE: &str = "BitNode-Console:API";

/// JWT audience for refresh tokens (token renewal endpoint).
static REFRESH_AUDIENCE: &str = "BitNode-Console:Auth";

/// JWT claim payload carried inside access and refresh tokens.
///
/// Standard registered claims follow [RFC 7519 §4.1](https://www.rfc-editor.org/rfc/rfc7519#section-4.1).
/// `jty` is a private claim that records the token type so callers can
/// distinguish an access token from a refresh token after decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TokenClaim {
    /// Issuer — always [`TOKEN_ISSUER`].
    pub iss: String,
    /// Audience — `"BitNode-Console:API"` for access tokens, `"BitNode-Console:Auth"` for refresh tokens.
    pub aud: String,
    /// Expiration time (UTC Unix timestamp).
    pub exp: u64,
    /// Not-before time (UTC Unix timestamp).
    pub nbf: u64,
    /// Issued-at time (UTC Unix timestamp).
    pub iat: u64,
    /// Unique token identifier (UUID v7).
    pub jti: String,
    /// Token type — `"Access"` or `"Refresh"` (private claim).
    pub jty: String,
}

impl TokenClaim {
    /// Construct a new token claim for `subject` with the given type and validity window.
    ///
    /// `duration` is in seconds. Both `nbf` and `iat` are set to the current
    /// moment; `exp` is `duration` seconds from now.
    ///
    /// # Panics
    ///
    /// Panics if the system clock is set before the Unix epoch, or if adding
    /// `duration` to the current time overflows.
    #[tracing::instrument(fields(token_type = %token_type))]
    pub fn new(token_type: &TokenType, duration: u64) -> Self {
        let now = SystemTime::now();

        let unix_now = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_secs();

        let expiration = now
            .checked_add(Duration::from_secs(duration))
            .expect("expiration time overflowed")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("expiration before Unix epoch")
            .as_secs();

        let audience = match token_type {
            TokenType::Access => ACCESS_AUDIENCE,
            TokenType::Refresh => REFRESH_AUDIENCE,
        };

        Self {
            iss: TOKEN_ISSUER.to_owned(),
            aud: audience.to_owned(),
            exp: expiration,
            nbf: unix_now,
            iat: unix_now,
            jti: Uuid::now_v7().to_string(),
            jty: token_type.to_string(),
        }
    }

    /// Sign and encode the claim as a compact JWT string.
    ///
    /// # Errors
    ///
    /// Returns an error if signing with `secret` fails.
    #[tracing::instrument(skip(self, secret), fields(jty = %self.jty, jti = %self.jti))]
    pub fn encode(&self, secret: &SecretString) -> crate::Result<String> {
        let key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
        encode(&Header::new(Algorithm::HS256), self, &key).map_err(crate::Error::Token)
    }

    /// Decode and verify a JWT string, returning the embedded claim on success.
    ///
    /// Validates the signature, expiry (`exp`), not-before (`nbf`), and issuer
    /// (`iss`). Audience is not validated here because access and refresh tokens
    /// carry different audiences; check [`TokenClaim::jty`] when the caller
    /// must distinguish the two token types.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is invalid, the token has expired, or
    /// the `nbf`/`iss` claims fail validation.
    #[tracing::instrument(skip(token, secret))]
    pub fn from_token(token: &str, secret: &SecretString) -> crate::Result<Self> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[TOKEN_ISSUER]);
        validation.validate_nbf = true;
        // Audience differs between access and refresh tokens; callers must
        // check `jty` themselves after decoding.
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["iss", "exp", "nbf"]);

        decode::<Self>(
            token,
            &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| {
            tracing::warn!(error = %e, "token verification failed");
            crate::Error::Token(e)
        })
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{TOKEN_ISSUER, TokenClaim, TokenType};

    type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

    fn secret() -> SecretString {
        SecretString::from("test-secret-key-long-enough-for-hs256-32c")
    }

    #[test]
    fn round_trip_access_token() -> Result<()> {
        let claim = TokenClaim::new(&TokenType::Access, 300);
        let token = claim.encode(&secret())?;
        let decoded = TokenClaim::from_token(&token, &secret())?;

        assert_eq!(decoded.iss, TOKEN_ISSUER);
        assert_eq!(decoded.jty, TokenType::Access.to_string());
        assert_eq!(decoded.aud, "BitNode-Console:API");
        Ok(())
    }

    #[test]
    fn round_trip_refresh_token() -> Result<()> {
        let claim = TokenClaim::new(&TokenType::Refresh, 7200);
        let token = claim.encode(&secret())?;
        let decoded = TokenClaim::from_token(&token, &secret())?;

        assert_eq!(decoded.jty, TokenType::Refresh.to_string());
        assert_eq!(decoded.aud, "BitNode-Console:Auth");
        Ok(())
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let claim = TokenClaim::new(&TokenType::Access, 300);
        let token = claim.encode(&secret()).unwrap();
        let wrong = SecretString::from("a-completely-different-secret-key-32c");

        assert!(TokenClaim::from_token(&token, &wrong).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let mut claim = TokenClaim::new(&TokenType::Access, 300);
        // Force expiry to a date well in the past (2001-09-09 UTC).
        claim.exp = 1_000_000_000;
        let token = claim.encode(&secret()).unwrap();

        assert!(TokenClaim::from_token(&token, &secret()).is_err());
    }
}
