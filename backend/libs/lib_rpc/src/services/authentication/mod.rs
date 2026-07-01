//! gRPC Authentication service — delegates each RPC to its own handler module.

mod login;
mod logout;
mod refresh;
mod service_impl;

pub use crate::generated_protos::authentication::authentication_service_server::{
    AuthenticationService, AuthenticationServiceServer,
};
pub use crate::generated_protos::authentication::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RefreshRequest, RefreshResponse,
};
pub use service_impl::AuthenticationServiceImpl;

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
