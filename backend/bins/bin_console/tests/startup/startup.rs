//! Integration tests for startup validation.
//!
//! The empty-password-hash guard lives in [`lib_rpc::Server::run`]; these
//! tests verify it fires correctly before any service begins accepting requests.

const VALID_HASH: &str =
    "$argon2id$v=19$m=4096,t=3,p=1$Mzk3NGQ1NDBiZmZiYjhhNDY0YWY3MmRjYjIwNDI2YmE\
     $Bzo73djW0DkCVR4prXhqvWB4ViEdJ54h91mU0LCgqPY";

fn rpc_settings(password_hash: &str) -> lib_settings::RpcSettings {
    lib_settings::RpcSettings {
        host: "127.0.0.1".to_string(),
        port: 0,
        password_hash: password_hash.to_string(),
        token_secret: "test_secret".to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn empty_password_hash_returns_config_error() {
    let server = lib_rpc::Server::new(rpc_settings("")).await.unwrap();

    let result = server.run().await;

    assert!(
        matches!(result, Err(lib_rpc::Error::Config(_))),
        "expected Config error, got {result:?}"
    );
}

#[tokio::test]
async fn non_empty_password_hash_starts_server() {
    let server = lib_rpc::Server::new(rpc_settings(VALID_HASH)).await.unwrap();
    let addr = server.address().unwrap();
    tokio::spawn(server.run());

    let mut client =
        lib_rpc::UtilitiesServiceClient::connect(format!("http://{addr}")).await.unwrap();
    let response =
        client.ping(tonic::Request::new(lib_rpc::PingRequest {})).await.unwrap();

    assert_eq!(response.into_inner().pong, "Pong...");
}
