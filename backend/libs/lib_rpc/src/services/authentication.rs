//! Implementation of the gRPC Authentication service.

use secrecy::SecretString;

pub use crate::generated_protos::authentication::authentication_service_server::{
    AuthenticationService, AuthenticationServiceServer,
};

pub use crate::generated_protos::authentication::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RefreshRequest, RefreshResponse,
};

/// Concrete implementation of the [`AuthenticationService`] gRPC trait.
#[derive(Debug, Clone)]
pub struct AuthenticationServiceImpl {
    password_hash: lib_auth::PasswordHash,
    token_secret: SecretString,
}

impl AuthenticationServiceImpl {
    /// Create a new [`AuthenticationServiceImpl`] that verifies login passwords
    /// against `password_hash` and signs tokens with `token_secret`.
    pub fn new(password_hash: lib_auth::PasswordHash, token_secret: SecretString) -> Self {
        Self {
            password_hash,
            token_secret,
        }
    }
}

#[tonic::async_trait]
impl AuthenticationService for AuthenticationServiceImpl {
    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> std::result::Result<tonic::Response<LoginResponse>, tonic::Status> {
        tracing::debug!("Login request received");

        //-- Extract the login request from the incoming request
        let login_request = request.into_inner();

        //-- Validate the login request
        if login_request.password.is_empty() {
            tracing::warn!("Login rejected: password field is empty");
            return Err(tonic::Status::invalid_argument(
                "password must not be empty",
            ));
        }

        //-- Extract the password from the login request
        let password = SecretString::from(login_request.password);

        //-- Verify the password against the stored hash
        let verified = self.password_hash.verify_password(&password).map_err(|e| {
            tracing::error!("Password hash error: {e}");
            tonic::Status::internal("authentication error")
        })?;

        //-- If the password is not verified, reject the login request
        if !verified {
            tracing::warn!("Login rejected: invalid password");
            return Err(tonic::Status::unauthenticated("invalid password"));
        }

        //-- Generate access and refresh tokens
        let access_token = lib_auth::AccessToken::new(&self.token_secret).map_err(|e| {
            tracing::error!("Failed to generate access token: {e}");
            tonic::Status::internal("authentication error")
        })?;

        let refresh_token = lib_auth::RefreshToken::new(&self.token_secret).map_err(|e| {
            tracing::error!("Failed to generate refresh token: {e}");
            tonic::Status::internal("authentication error")
        })?;

        tracing::info!("Login successful; tokens issued");

        //-- Return the tokens to the client
        Ok(tonic::Response::new(LoginResponse {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        }))
    }

    async fn refresh(
        &self,
        request: tonic::Request<RefreshRequest>,
    ) -> std::result::Result<tonic::Response<RefreshResponse>, tonic::Status> {
        tracing::debug!("Refresh request received from {:?}", request.remote_addr());

        let refresh_request = request.into_inner();

        //-- Guard against an empty token field
        if refresh_request.refresh_token.is_empty() {
            tracing::warn!("Refresh rejected: token field is empty");
            return Err(tonic::Status::invalid_argument(
                "refresh token must not be empty",
            ));
        }

        //-- Wrap and validate the refresh token (checks signature, expiry, and token type)
        let refresh_token = lib_auth::RefreshToken::from(refresh_request.refresh_token);
        refresh_token.validate(&self.token_secret).map_err(|e| {
            tracing::warn!("Refresh token validation failed: {e}");
            tonic::Status::unauthenticated("invalid or expired refresh token")
        })?;

        //-- Issue a new access token
        let access_token = lib_auth::AccessToken::new(&self.token_secret).map_err(|e| {
            tracing::error!("Failed to generate access token: {e}");
            tonic::Status::internal("authentication error")
        })?;

        //-- Issue a new refresh token (token rotation — extends the session window and limits
        //-- replay exposure if the old token is ever intercepted)
        let new_refresh_token = lib_auth::RefreshToken::new(&self.token_secret).map_err(|e| {
            tracing::error!("Failed to generate refresh token: {e}");
            tonic::Status::internal("authentication error")
        })?;

        tracing::info!("Token refresh successful; new token pair issued");

        Ok(tonic::Response::new(RefreshResponse {
            access_token: access_token.to_string(),
            refresh_token: new_refresh_token.to_string(),
        }))
    }

    async fn logout(
        &self,
        request: tonic::Request<LogoutRequest>,
    ) -> std::result::Result<tonic::Response<LogoutResponse>, tonic::Status> {
        tracing::debug!("Logout request received from {:?}", request.remote_addr());
        tracing::info!("Token invalidation not yet implemented");
        Err(tonic::Status::unimplemented(
            "logout is not yet implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    const TEST_PASSWORD: &str = "test_password";
    const WRONG_PASSWORD: &str = "wrong_password";
    const TEST_SECRET: &str = "test-secret-key-long-enough-for-hs256-32c";

    /// Compute the Argon2id hash of TEST_PASSWORD once for the entire test run.
    static HASH: OnceLock<lib_auth::PasswordHash> = OnceLock::new();

    fn test_hash() -> &'static lib_auth::PasswordHash {
        HASH.get_or_init(|| {
            lib_auth::PasswordHash::from_password(&SecretString::from(TEST_PASSWORD))
                .expect("test hash must compute")
        })
    }

    fn service() -> AuthenticationServiceImpl {
        AuthenticationServiceImpl::new(test_hash().clone(), SecretString::from(TEST_SECRET))
    }

    fn login_request(password: &str) -> tonic::Request<LoginRequest> {
        tonic::Request::new(LoginRequest {
            password: password.to_string(),
        })
    }

    fn refresh_request(token: &str) -> tonic::Request<RefreshRequest> {
        tonic::Request::new(RefreshRequest {
            refresh_token: token.to_string(),
        })
    }

    fn logout_request() -> tonic::Request<LogoutRequest> {
        tonic::Request::new(LogoutRequest {})
    }

    // --- login: empty password guard ---

    #[tokio::test]
    async fn login_rejects_empty_password() {
        let status = service().login(login_request("")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn login_empty_password_error_message() {
        let status = service().login(login_request("")).await.unwrap_err();
        assert_eq!(status.message(), "password must not be empty");
    }

    // --- login: wrong password ---

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let status = service().login(login_request(WRONG_PASSWORD)).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn login_wrong_password_error_message() {
        let status = service().login(login_request(WRONG_PASSWORD)).await.unwrap_err();
        assert_eq!(status.message(), "invalid password");
    }

    #[tokio::test]
    async fn login_rejects_whitespace_only_password() {
        let status = service().login(login_request("   ")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn login_rejects_partial_password() {
        let status = service().login(login_request("test_pass")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    // --- login: correct password ---

    #[tokio::test]
    async fn login_correct_password_returns_ok() {
        let result = service().login(login_request(TEST_PASSWORD)).await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[tokio::test]
    async fn login_correct_password_returns_non_empty_access_token() {
        let response = service().login(login_request(TEST_PASSWORD)).await.unwrap();
        assert!(!response.into_inner().access_token.is_empty());
    }

    #[tokio::test]
    async fn login_correct_password_returns_non_empty_refresh_token() {
        let response = service().login(login_request(TEST_PASSWORD)).await.unwrap();
        assert!(!response.into_inner().refresh_token.is_empty());
    }

    #[tokio::test]
    async fn login_correct_password_returns_three_part_jwt_access_token() {
        let response = service().login(login_request(TEST_PASSWORD)).await.unwrap();
        let token = response.into_inner().access_token;
        assert_eq!(token.split('.').count(), 3, "access token must be a JWT");
    }

    #[tokio::test]
    async fn login_correct_password_returns_three_part_jwt_refresh_token() {
        let response = service().login(login_request(TEST_PASSWORD)).await.unwrap();
        let token = response.into_inner().refresh_token;
        assert_eq!(token.split('.').count(), 3, "refresh token must be a JWT");
    }

    // --- refresh ---

    fn valid_refresh_token() -> String {
        lib_auth::RefreshToken::new(&SecretString::from(TEST_SECRET))
            .expect("test refresh token must generate")
            .to_string()
    }

    #[tokio::test]
    async fn refresh_rejects_empty_token() {
        let status = service().refresh(refresh_request("")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn refresh_empty_token_error_message() {
        let status = service().refresh(refresh_request("")).await.unwrap_err();
        assert_eq!(status.message(), "refresh token must not be empty");
    }

    #[tokio::test]
    async fn refresh_rejects_invalid_token() {
        let status = service().refresh(refresh_request("not.a.real.jwt")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn refresh_invalid_token_error_message() {
        let status = service().refresh(refresh_request("not.a.real.jwt")).await.unwrap_err();
        assert_eq!(status.message(), "invalid or expired refresh token");
    }

    #[tokio::test]
    async fn refresh_valid_token_returns_ok() {
        let result = service().refresh(refresh_request(&valid_refresh_token())).await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[tokio::test]
    async fn refresh_returns_non_empty_access_token() {
        let response = service().refresh(refresh_request(&valid_refresh_token())).await.unwrap();
        assert!(!response.into_inner().access_token.is_empty());
    }

    #[tokio::test]
    async fn refresh_returns_non_empty_refresh_token() {
        let response = service().refresh(refresh_request(&valid_refresh_token())).await.unwrap();
        assert!(!response.into_inner().refresh_token.is_empty());
    }

    #[tokio::test]
    async fn refresh_returns_three_part_jwt_access_token() {
        let response = service().refresh(refresh_request(&valid_refresh_token())).await.unwrap();
        let token = response.into_inner().access_token;
        assert_eq!(token.split('.').count(), 3, "access token must be a JWT");
    }

    #[tokio::test]
    async fn refresh_returns_three_part_jwt_refresh_token() {
        let response = service().refresh(refresh_request(&valid_refresh_token())).await.unwrap();
        let token = response.into_inner().refresh_token;
        assert_eq!(token.split('.').count(), 3, "refresh token must be a JWT");
    }

    #[tokio::test]
    async fn refresh_issues_new_refresh_token_each_time() {
        let token = valid_refresh_token();
        let first = service()
            .refresh(refresh_request(&token))
            .await
            .unwrap()
            .into_inner()
            .refresh_token;
        let second = service()
            .refresh(refresh_request(&token))
            .await
            .unwrap()
            .into_inner()
            .refresh_token;
        assert_ne!(first, second, "each refresh should produce a unique token");
    }

    // --- logout ---

    #[tokio::test]
    async fn logout_returns_unimplemented() {
        let status = service().logout(logout_request()).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unimplemented);
    }

    #[tokio::test]
    async fn logout_unimplemented_error_message() {
        let status = service().logout(logout_request()).await.unwrap_err();
        assert_eq!(status.message(), "logout is not yet implemented");
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

    #[test]
    fn refresh_response_stores_tokens() {
        let resp = RefreshResponse {
            access_token: "new_access".to_string(),
            refresh_token: "new_refresh".to_string(),
        };
        assert_eq!(resp.access_token, "new_access");
        assert_eq!(resp.refresh_token, "new_refresh");
    }

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
    fn logout_request_default_is_empty() {
        assert_eq!(LogoutRequest::default(), LogoutRequest {});
    }

    #[test]
    fn logout_response_default_is_empty() {
        assert_eq!(LogoutResponse::default(), LogoutResponse {});
    }

    // --- service impl ---

    #[test]
    fn service_impl_has_debug() {
        assert!(format!("{:?}", service()).contains("AuthenticationServiceImpl"));
    }

    #[test]
    fn authentication_service_server_wraps_impl() {
        let _server = AuthenticationServiceServer::new(service());
    }
}
