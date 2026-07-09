//! Refresh handler for the Authentication gRPC service.

use secrecy::SecretString;

use crate::services::authentication::{RefreshRequest, RefreshResponse};

/// Handle a refresh request — validate the refresh token and issue a new token pair.
///
/// Issues a new refresh token alongside the new access token (token rotation). Each
/// redemption produces a unique pair, limiting the replay window if an old token is
/// ever intercepted. Note: without a token store, the redeemed token remains valid
/// until its own expiry — full revocation requires persisting issued JTIs.
#[tracing::instrument(skip_all)]
pub(super) async fn handle(
    token_secret: &SecretString,
    request: tonic::Request<RefreshRequest>,
) -> crate::Result<tonic::Response<RefreshResponse>> {
    tracing::debug!("Refresh request received from {:?}", request.remote_addr());

    let refresh_request = request.into_inner();

    if refresh_request.refresh_token.is_empty() {
        tracing::warn!("Refresh rejected: token field is empty");
        return Err(crate::Error::InvalidArgument(
            "refresh token must not be empty".to_string(),
        ));
    }

    let refresh_token = lib_auth::RefreshToken::from(refresh_request.refresh_token);
    refresh_token.validate(token_secret).map_err(|e| {
        tracing::warn!("Refresh token validation failed: {e}");
        crate::Error::Authentication("invalid or expired refresh token".to_string())
    })?;

    let access_token = lib_auth::AccessToken::new(token_secret).map_err(|e| {
        tracing::error!("Failed to generate access token: {e}");
        crate::Error::Generic("authentication error".to_string())
    })?;

    let new_refresh_token = lib_auth::RefreshToken::new(token_secret).map_err(|e| {
        tracing::error!("Failed to generate refresh token: {e}");
        crate::Error::Generic("authentication error".to_string())
    })?;

    tracing::info!("Token refresh successful; new token pair issued");

    Ok(tonic::Response::new(RefreshResponse {
        access_token: access_token.to_string(),
        refresh_token: new_refresh_token.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{RefreshRequest, RefreshResponse, handle};

    const TEST_SECRET: &str = "test-secret-key-long-enough-for-hs256-32c";

    fn secret() -> SecretString {
        SecretString::from(TEST_SECRET)
    }

    fn refresh_request(token: &str) -> tonic::Request<RefreshRequest> {
        tonic::Request::new(RefreshRequest {
            refresh_token: token.to_string(),
        })
    }

    fn valid_refresh_token() -> String {
        lib_auth::RefreshToken::new(&secret())
            .expect("test refresh token must generate")
            .to_string()
    }

    // --- empty token guard ---

    #[tokio::test]
    async fn refresh_rejects_empty_token() {
        let status = tonic::Status::from(handle(&secret(), refresh_request("")).await.unwrap_err());
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn refresh_empty_token_error_message() {
        let status = tonic::Status::from(handle(&secret(), refresh_request("")).await.unwrap_err());
        assert_eq!(status.message(), "refresh token must not be empty");
    }

    // --- invalid token ---

    #[tokio::test]
    async fn refresh_rejects_invalid_token() {
        let status = tonic::Status::from(
            handle(&secret(), refresh_request("not.a.real.jwt")).await.unwrap_err(),
        );
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn refresh_invalid_token_error_message() {
        let status = tonic::Status::from(
            handle(&secret(), refresh_request("not.a.real.jwt")).await.unwrap_err(),
        );
        assert_eq!(status.message(), "invalid or expired refresh token");
    }

    // --- valid token ---

    #[tokio::test]
    async fn refresh_valid_token_returns_ok() {
        let result = handle(&secret(), refresh_request(&valid_refresh_token())).await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[tokio::test]
    async fn refresh_returns_non_empty_access_token() {
        let response = handle(&secret(), refresh_request(&valid_refresh_token())).await.unwrap();
        assert!(!response.into_inner().access_token.is_empty());
    }

    #[tokio::test]
    async fn refresh_returns_non_empty_refresh_token() {
        let response = handle(&secret(), refresh_request(&valid_refresh_token())).await.unwrap();
        assert!(!response.into_inner().refresh_token.is_empty());
    }

    #[tokio::test]
    async fn refresh_returns_three_part_jwt_access_token() {
        let response = handle(&secret(), refresh_request(&valid_refresh_token())).await.unwrap();
        let token = response.into_inner().access_token;
        assert_eq!(token.split('.').count(), 3, "access token must be a JWT");
    }

    #[tokio::test]
    async fn refresh_returns_three_part_jwt_refresh_token() {
        let response = handle(&secret(), refresh_request(&valid_refresh_token())).await.unwrap();
        let token = response.into_inner().refresh_token;
        assert_eq!(token.split('.').count(), 3, "refresh token must be a JWT");
    }

    #[tokio::test]
    async fn refresh_issues_new_refresh_token_each_time() {
        let token = valid_refresh_token();
        let first = handle(&secret(), refresh_request(&token))
            .await
            .unwrap()
            .into_inner()
            .refresh_token;
        let second = handle(&secret(), refresh_request(&token))
            .await
            .unwrap()
            .into_inner()
            .refresh_token;
        assert_ne!(first, second, "each refresh should produce a unique token");
    }

    // --- message construction ---

    #[test]
    fn refresh_request_default_is_empty() {
        assert_eq!(
            RefreshRequest::default(),
            RefreshRequest {
                refresh_token: String::new()
            }
        );
    }

    #[test]
    fn refresh_response_stores_tokens() {
        let resp = RefreshResponse {
            access_token: "new_access".to_string(),
            refresh_token: "new_refresh".to_string(),
        };
        assert_eq!(resp.access_token, "new_access");
        assert_eq!(resp.refresh_token, "new_refresh");
    }
}
