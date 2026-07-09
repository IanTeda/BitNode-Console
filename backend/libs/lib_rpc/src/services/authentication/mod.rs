//! gRPC Authentication service — delegates each RPC to its own handler module.

mod login;
mod logout;
mod refresh;
mod service_impl;

pub use crate::generated_protos::authentication::authentication_service_client::AuthenticationServiceClient;
pub use crate::generated_protos::authentication::authentication_service_server::{
    AuthenticationService, AuthenticationServiceServer,
};
pub use crate::generated_protos::authentication::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RefreshRequest, RefreshResponse,
};
pub use service_impl::AuthenticationServiceImpl;
