use secrecy::SecretString;

use crate::services::authentication::{
    AuthenticationService, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
    RefreshRequest, RefreshResponse,
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
        super::login::handle(&self.password_hash, &self.token_secret, request)
            .await
            .map_err(Into::into)
    }

    async fn refresh(
        &self,
        request: tonic::Request<RefreshRequest>,
    ) -> std::result::Result<tonic::Response<RefreshResponse>, tonic::Status> {
        super::refresh::handle(&self.token_secret, request)
            .await
            .map_err(Into::into)
    }

    async fn logout(
        &self,
        request: tonic::Request<LogoutRequest>,
    ) -> std::result::Result<tonic::Response<LogoutResponse>, tonic::Status> {
        super::logout::handle(request).await.map_err(Into::into)
    }
}
