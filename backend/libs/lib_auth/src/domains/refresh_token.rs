//! JWT refresh token for authorising the issuance of a new access token.

use secrecy::SecretString;

use super::{TokenClaim, TokenType};

/// Validity window for refresh tokens (2 hours).
pub static REFRESH_TOKEN_DURATION: u64 = 2 * 60 * 60;

/// A signed JWT that authorises the issuance of a new [`AccessToken`].
///
/// Construct via [`RefreshToken::new`] and store on the client (e.g. an
/// `HttpOnly` cookie). When the access token expires, present the refresh token
/// to the token-renewal endpoint and verify it via [`RefreshToken::from_token`].
///
/// A stored token string (e.g. read back from a database or cookie) can be
/// wrapped with [`RefreshToken::from`] before being passed around as a typed
/// value.
///
/// [`AccessToken`]: super::AccessToken
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RefreshToken(String);

impl AsRef<str> for RefreshToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Wrap a raw token string (e.g. read from a database or cookie) as a typed
/// [`RefreshToken`] without performing any validation.
impl From<String> for RefreshToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Wrap a borrowed token string slice (e.g. extracted from a request header or
/// cookie) as a typed [`RefreshToken`] without performing any validation.
impl From<&str> for RefreshToken {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl RefreshToken {
    /// Generate a new refresh token signed with `secret`.
    ///
    /// The embedded claim expires after [`REFRESH_TOKEN_DURATION`] seconds.
    #[tracing::instrument(skip(secret))]
    pub fn new(secret: &SecretString) -> crate::Result<Self> {
        let claim = TokenClaim::new(&TokenType::Refresh, REFRESH_TOKEN_DURATION);
        let token = claim.encode(secret)?;
        tracing::debug!(jti = %claim.jti, "refresh token issued");
        Ok(Self(token))
    }

    /// Decode and verify a refresh token string, returning its embedded claim.
    ///
    /// Returns an error if the signature is invalid, the token has expired, the
    /// `nbf` or `iss` claims fail validation, or the token is not a refresh
    /// token (e.g. an access token was supplied in its place).
    #[tracing::instrument(skip(token, secret))]
    pub fn from_token(token: &str, secret: &SecretString) -> crate::Result<TokenClaim> {
        let claim = TokenClaim::from_token(token, secret)?;

        if claim.jty != TokenType::Refresh.to_string() {
            tracing::warn!(jty = %claim.jty, expected = %TokenType::Refresh, "token type mismatch");
            return Err(crate::Error::InvalidTokenType {
                expected: TokenType::Refresh.to_string(),
                got: claim.jty,
            });
        }

        Ok(claim)
    }

    /// Decode and verify this refresh token, returning its embedded claim.
    ///
    /// Equivalent to [`RefreshToken::from_token`] called with `self` as the
    /// token string — use this when you already hold a typed [`RefreshToken`]
    /// instance and want to verify it without extracting the inner string
    /// manually.
    #[tracing::instrument(skip(self, secret))]
    pub fn validate(&self, secret: &SecretString) -> crate::Result<TokenClaim> {
        Self::from_token(self.as_ref(), secret)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{REFRESH_TOKEN_DURATION, RefreshToken};
    use crate::domains::{TokenClaim, TokenType};

    type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

    fn secret() -> SecretString {
        SecretString::from("test-secret-key-long-enough-for-hs256-32c")
    }

    fn wrong_secret() -> SecretString {
        SecretString::from("a-completely-different-secret-key-32c")
    }

    #[test]
    fn new_produces_three_part_jwt() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        assert_eq!(token.as_ref().split('.').count(), 3);
        Ok(())
    }

    #[test]
    fn from_token_returns_refresh_claim() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        let claim = RefreshToken::from_token(token.as_ref(), &secret())?;

        assert_eq!(claim.jty, TokenType::Refresh.to_string());
        assert_eq!(claim.aud, "BitNode-Console:Auth");
        Ok(())
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token = RefreshToken::new(&secret()).unwrap();
        assert!(RefreshToken::from_token(token.as_ref(), &wrong_secret()).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let mut claim = TokenClaim::new(&TokenType::Refresh, 7200);
        claim.exp = 1_000_000_000; // 2001-09-09 UTC — definitely in the past
        let token = claim.encode(&secret()).unwrap();
        assert!(RefreshToken::from_token(&token, &secret()).is_err());
    }

    #[test]
    fn access_token_is_rejected_as_refresh() {
        let claim = TokenClaim::new(&TokenType::Access, 300);
        let token = claim.encode(&secret()).unwrap();
        let err = RefreshToken::from_token(&token, &secret()).unwrap_err();
        assert!(matches!(err, crate::Error::InvalidTokenType { .. }));
    }

    #[test]
    fn validate_returns_refresh_claim() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        let claim = token.validate(&secret())?;

        assert_eq!(claim.jty, TokenType::Refresh.to_string());
        Ok(())
    }

    #[test]
    fn validate_rejects_wrong_secret() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        assert!(token.validate(&wrong_secret()).is_err());
        Ok(())
    }

    #[test]
    fn validate_rejects_empty_token() {
        let token = RefreshToken::default();
        assert!(token.validate(&secret()).is_err());
    }

    #[test]
    fn validate_rejects_access_token_as_refresh() -> Result<()> {
        let claim = TokenClaim::new(&TokenType::Access, 300);
        let raw = claim.encode(&secret())?;
        let token = RefreshToken::from(raw);
        let err = token.validate(&secret()).unwrap_err();
        assert!(matches!(err, crate::Error::InvalidTokenType { .. }));
        Ok(())
    }

    #[test]
    fn from_string_wraps_without_validation() {
        let raw = "not.a.real.jwt".to_string();
        let token = RefreshToken::from(raw.clone());
        assert_eq!(token.as_ref(), raw);
    }

    #[test]
    fn from_str_wraps_without_validation() {
        let raw = "not.a.real.jwt";
        let token = RefreshToken::from(raw);
        assert_eq!(token.as_ref(), raw);
    }

    #[test]
    fn from_string_then_verify_rejects_invalid_token() {
        let token = RefreshToken::from("not.a.real.jwt".to_string());
        assert!(RefreshToken::from_token(token.as_ref(), &secret()).is_err());
    }

    #[test]
    fn display_matches_as_ref() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        assert_eq!(token.to_string(), token.as_ref());
        Ok(())
    }

    #[test]
    fn clone_produces_equal_value() -> Result<()> {
        let token = RefreshToken::new(&secret())?;
        assert_eq!(token.clone(), token);
        Ok(())
    }

    #[test]
    fn default_is_empty() {
        assert_eq!(RefreshToken::default().as_ref(), "");
    }

    #[test]
    fn duration_is_two_hours() {
        assert_eq!(REFRESH_TOKEN_DURATION, 7200);
    }
}
