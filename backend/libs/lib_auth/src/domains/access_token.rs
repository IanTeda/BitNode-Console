//! JWT access token for authorising RPC endpoint requests.

use secrecy::SecretString;

use super::{TokenClaim, TokenType};

/// Validity window for access tokens (5 minutes).
pub static ACCESS_TOKEN_DURATION: u64 = 5 * 60;

/// A signed JWT that authorises a single RPC request.
///
/// Construct via [`AccessToken::new`] and forward to the client. On the
/// receiving end, decode and verify via [`AccessToken::from_token`] to obtain
/// the embedded [`TokenClaim`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessToken(String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AccessToken {
    /// Generate a new access token signed with `secret`.
    ///
    /// The embedded claim expires after [`ACCESS_TOKEN_DURATION`] seconds.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding the JWT with `secret` fails.
    #[tracing::instrument(skip(secret))]
    pub fn new(secret: &SecretString) -> crate::Result<Self> {
        let claim = TokenClaim::new(&TokenType::Access, ACCESS_TOKEN_DURATION);
        let token = claim.encode(secret)?;
        tracing::debug!(jti = %claim.jti, "access token issued");
        Ok(Self(token))
    }

    /// Decode and verify an access token string, returning its embedded claim.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is invalid, the token has expired, the
    /// `nbf` or `iss` claims fail validation, or the token is not an access token
    /// (e.g. a refresh token was supplied in its place).
    #[tracing::instrument(skip(token, secret))]
    pub fn from_token(token: &str, secret: &SecretString) -> crate::Result<TokenClaim> {
        let claim = TokenClaim::from_token(token, secret)?;

        if claim.jty != TokenType::Access.to_string() {
            tracing::warn!(jty = %claim.jty, expected = %TokenType::Access, "token type mismatch");
            return Err(crate::Error::InvalidTokenType {
                expected: TokenType::Access.to_string(),
                got: claim.jty,
            });
        }

        Ok(claim)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{ACCESS_TOKEN_DURATION, AccessToken};
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
        let token = AccessToken::new(&secret())?;
        assert_eq!(token.as_ref().split('.').count(), 3);
        Ok(())
    }

    #[test]
    fn from_token_returns_access_claim() -> Result<()> {
        let token = AccessToken::new(&secret())?;
        let claim = AccessToken::from_token(token.as_ref(), &secret())?;

        assert_eq!(claim.jty, TokenType::Access.to_string());
        assert_eq!(claim.aud, "BitNode-Console:API");
        Ok(())
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token = AccessToken::new(&secret()).unwrap();
        assert!(AccessToken::from_token(token.as_ref(), &wrong_secret()).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let mut claim = TokenClaim::new(&TokenType::Access, 300);
        claim.exp = 1_000_000_000; // 2001-09-09 UTC — definitely in the past
        let token = claim.encode(&secret()).unwrap();
        assert!(AccessToken::from_token(&token, &secret()).is_err());
    }

    #[test]
    fn refresh_token_is_rejected_as_access() {
        let claim = TokenClaim::new(&TokenType::Refresh, 7200);
        let token = claim.encode(&secret()).unwrap();
        let err = AccessToken::from_token(&token, &secret()).unwrap_err();
        assert!(matches!(err, crate::Error::InvalidTokenType { .. }));
    }

    #[test]
    fn display_matches_as_ref() -> Result<()> {
        let token = AccessToken::new(&secret())?;
        assert_eq!(token.to_string(), token.as_ref());
        Ok(())
    }

    #[test]
    fn clone_produces_equal_value() -> Result<()> {
        let token = AccessToken::new(&secret())?;
        assert_eq!(token.clone(), token);
        Ok(())
    }

    #[test]
    fn default_is_empty() {
        assert_eq!(AccessToken::default().as_ref(), "");
    }

    #[test]
    fn duration_is_five_minutes() {
        assert_eq!(ACCESS_TOKEN_DURATION, 300);
    }
}
