//! Access-token interceptor for gRPC request authentication.

use secrecy::SecretString;

/// Interceptor that validates the access token on every incoming RPC request.
///
/// Reads the JWT from the `access_token` gRPC metadata header. If the header is
/// absent or the token is invalid (bad signature, expired, or wrong type), the
/// request is rejected with [`tonic::Code::Unauthenticated`].
///
/// On success the decoded [`lib_auth::TokenClaim`] is inserted into the request
/// extensions so downstream handlers can read it without re-decoding.
#[derive(Clone)]
pub struct AccessTokenInterceptor {
    token_secret: SecretString,
}

impl AccessTokenInterceptor {
    /// Create a new [`AccessTokenInterceptor`] that verifies tokens signed with `token_secret`.
    pub fn new(token_secret: SecretString) -> Self {
        Self { token_secret }
    }
}

impl tonic::service::Interceptor for AccessTokenInterceptor {
    #[tracing::instrument(name = "access_token.call", skip_all, level = "debug")]
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let token_str = request
            .metadata()
            .get("access_token")
            .ok_or_else(|| crate::Error::Authentication("missing access_token header".to_string()))?
            .to_str()
            .map_err(|_| {
                crate::Error::Authentication(
                    "access_token header contains non-ASCII bytes".to_string(),
                )
            })?;

        let claim = lib_auth::AccessToken::from_token(token_str, &self.token_secret)
            .map_err(|e| crate::Error::Authentication(e.to_string()))?;

        tracing::debug!(jti = %claim.jti, "access token verified");

        request.extensions_mut().insert(claim);

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use tonic::service::Interceptor as _;

    use super::AccessTokenInterceptor;

    const TEST_SECRET: &str = "test-secret-key-long-enough-for-hs256-32c";

    fn secret() -> SecretString {
        SecretString::from(TEST_SECRET)
    }

    fn interceptor() -> AccessTokenInterceptor {
        AccessTokenInterceptor::new(secret())
    }

    fn empty_request() -> tonic::Request<()> {
        tonic::Request::new(())
    }

    fn request_with_token(token: &str) -> tonic::Request<()> {
        let mut request = tonic::Request::new(());
        request.metadata_mut().insert(
            "access_token",
            token.parse().expect("token must be valid ASCII"),
        );
        request
    }

    fn valid_access_token() -> String {
        lib_auth::AccessToken::new(&secret())
            .expect("test access token must generate")
            .to_string()
    }

    // --- missing header ---

    #[test]
    fn rejects_request_without_access_token_header() {
        assert!(interceptor().call(empty_request()).is_err());
    }

    #[test]
    fn missing_header_returns_unauthenticated() {
        let err = interceptor().call(empty_request()).unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // --- invalid token ---

    #[test]
    fn rejects_malformed_jwt() {
        let err = interceptor().call(request_with_token("not.a.real.jwt")).unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn rejects_token_signed_with_wrong_secret() {
        let wrong_secret = SecretString::from("a-completely-different-secret-key-32c");
        let token = lib_auth::AccessToken::new(&wrong_secret)
            .expect("token must generate")
            .to_string();
        let err = interceptor().call(request_with_token(&token)).unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn rejects_refresh_token_presented_as_access_token() {
        let token = lib_auth::RefreshToken::new(&secret())
            .expect("refresh token must generate")
            .to_string();
        let err = interceptor().call(request_with_token(&token)).unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // --- valid token ---

    #[test]
    fn accepts_valid_access_token() {
        assert!(interceptor().call(request_with_token(&valid_access_token())).is_ok());
    }

    #[test]
    fn valid_token_inserts_claim_into_extensions() {
        let request = interceptor().call(request_with_token(&valid_access_token())).unwrap();
        assert!(
            request.extensions().get::<lib_auth::TokenClaim>().is_some(),
            "TokenClaim must be present in request extensions after successful auth"
        );
    }

    #[test]
    fn valid_token_claim_has_expected_token_type() {
        let request = interceptor().call(request_with_token(&valid_access_token())).unwrap();
        let claim = request.extensions().get::<lib_auth::TokenClaim>().unwrap();
        assert_eq!(claim.jty, lib_auth::TokenType::Access.to_string());
    }

    // --- Clone ---

    #[test]
    fn interceptor_is_clone() {
        let _copy = interceptor().clone();
    }
}
