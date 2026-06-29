//! gRPC Authentication service — delegates each RPC to its own handler module.

use secrecy::SecretString;

mod login;
mod logout;
mod refresh;

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
        Self { password_hash, token_secret }
    }
}

#[tonic::async_trait]
impl AuthenticationService for AuthenticationServiceImpl {
    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> std::result::Result<tonic::Response<LoginResponse>, tonic::Status> {
        login::handle(&self.password_hash, &self.token_secret, request).await
    }

    async fn refresh(
        &self,
        request: tonic::Request<RefreshRequest>,
    ) -> std::result::Result<tonic::Response<RefreshResponse>, tonic::Status> {
        refresh::handle(&self.token_secret, request).await
    }

    async fn logout(
        &self,
        request: tonic::Request<LogoutRequest>,
    ) -> std::result::Result<tonic::Response<LogoutResponse>, tonic::Status> {
        logout::handle(request).await
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use std::sync::OnceLock;

    use super::{AuthenticationService, AuthenticationServiceImpl, AuthenticationServiceServer};

    const TEST_PASSWORD: &str = "test_password";
    const TEST_SECRET: &str = "test-secret-key-long-enough-for-hs256-32c";

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

    #[test]
    fn service_impl_has_debug() {
        assert!(format!("{:?}", service()).contains("AuthenticationServiceImpl"));
    }

    #[test]
    fn authentication_service_server_wraps_impl() {
        let _server = AuthenticationServiceServer::new(service());
    }
}
