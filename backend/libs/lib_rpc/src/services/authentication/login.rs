//! Login handler for the Authentication gRPC service.

use secrecy::SecretString;

use crate::services::authentication::{LoginRequest, LoginResponse};

/// Handle a login request — verify the password and issue an access/refresh token pair.
#[tracing::instrument(skip_all)]
pub(super) async fn handle(
    password_hash: &lib_auth::PasswordHash,
    token_secret: &SecretString,
    request: tonic::Request<LoginRequest>,
) -> crate::Result<tonic::Response<LoginResponse>> {
    tracing::debug!("Login request received from {:?}", request.remote_addr());

    let login_request = request.into_inner();

    if login_request.password.is_empty() {
        tracing::warn!("Login rejected: password field is empty");
        return Err(crate::Error::InvalidArgument(
            "password must not be empty".to_string(),
        ));
    }

    let password = SecretString::from(login_request.password);

    let verified = password_hash.verify_password(&password).map_err(|e| {
        tracing::error!("Password hash error: {e}");
        crate::Error::Generic("authentication error".to_string())
    })?;

    if !verified {
        tracing::warn!("Login rejected: invalid password");
        return Err(crate::Error::Authentication("invalid password".to_string()));
    }

    let access_token = lib_auth::AccessToken::new(token_secret).map_err(|e| {
        tracing::error!("Failed to generate access token: {e}");
        crate::Error::Generic("authentication error".to_string())
    })?;

    let refresh_token = lib_auth::RefreshToken::new(token_secret).map_err(|e| {
        tracing::error!("Failed to generate refresh token: {e}");
        crate::Error::Generic("authentication error".to_string())
    })?;

    tracing::info!("Login successful; tokens issued");

    Ok(tonic::Response::new(LoginResponse {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use std::sync::OnceLock;

    use super::{LoginRequest, LoginResponse, handle};

    const TEST_PASSWORD: &str = "test_password";
    const WRONG_PASSWORD: &str = "wrong_password";
    const TEST_SECRET: &str = "test-secret-key-long-enough-for-hs256-32c";

    static HASH: OnceLock<lib_auth::PasswordHash> = OnceLock::new();

    fn test_hash() -> &'static lib_auth::PasswordHash {
        HASH.get_or_init(|| {
            lib_auth::PasswordHash::from_password(&SecretString::from(TEST_PASSWORD))
                .expect("test hash must compute")
        })
    }

    fn secret() -> SecretString {
        SecretString::from(TEST_SECRET)
    }

    fn login_request(password: &str) -> tonic::Request<LoginRequest> {
        tonic::Request::new(LoginRequest {
            password: password.to_string(),
        })
    }

    // --- empty password guard ---

    #[tokio::test]
    async fn login_rejects_empty_password() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request("")).await.unwrap_err(),
        );
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn login_empty_password_error_message() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request("")).await.unwrap_err(),
        );
        assert_eq!(status.message(), "password must not be empty");
    }

    // --- wrong password ---

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request(WRONG_PASSWORD)).await.unwrap_err(),
        );
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn login_wrong_password_error_message() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request(WRONG_PASSWORD)).await.unwrap_err(),
        );
        assert_eq!(status.message(), "invalid password");
    }

    #[tokio::test]
    async fn login_rejects_whitespace_only_password() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request("   ")).await.unwrap_err(),
        );
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn login_rejects_partial_password() {
        let status = tonic::Status::from(
            handle(test_hash(), &secret(), login_request("test_pass")).await.unwrap_err(),
        );
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    // --- correct password ---

    #[tokio::test]
    async fn login_correct_password_returns_ok() {
        let result = handle(test_hash(), &secret(), login_request(TEST_PASSWORD)).await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[tokio::test]
    async fn login_correct_password_returns_non_empty_access_token() {
        let response = handle(test_hash(), &secret(), login_request(TEST_PASSWORD)).await.unwrap();
        assert!(!response.into_inner().access_token.is_empty());
    }

    #[tokio::test]
    async fn login_correct_password_returns_non_empty_refresh_token() {
        let response = handle(test_hash(), &secret(), login_request(TEST_PASSWORD)).await.unwrap();
        assert!(!response.into_inner().refresh_token.is_empty());
    }

    #[tokio::test]
    async fn login_correct_password_returns_three_part_jwt_access_token() {
        let response = handle(test_hash(), &secret(), login_request(TEST_PASSWORD)).await.unwrap();
        let token = response.into_inner().access_token;
        assert_eq!(token.split('.').count(), 3, "access token must be a JWT");
    }

    #[tokio::test]
    async fn login_correct_password_returns_three_part_jwt_refresh_token() {
        let response = handle(test_hash(), &secret(), login_request(TEST_PASSWORD)).await.unwrap();
        let token = response.into_inner().refresh_token;
        assert_eq!(token.split('.').count(), 3, "refresh token must be a JWT");
    }

    // --- message construction ---

    #[test]
    fn login_request_stores_password() {
        let req = LoginRequest {
            password: "hunter2".to_string(),
        };
        assert_eq!(req.password, "hunter2");
    }

    #[test]
    fn login_request_default_has_empty_password() {
        assert!(LoginRequest::default().password.is_empty());
    }

    #[test]
    fn login_response_stores_tokens() {
        let resp = LoginResponse {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
        };
        assert_eq!(resp.access_token, "access");
        assert_eq!(resp.refresh_token, "refresh");
    }
}
