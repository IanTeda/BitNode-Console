//! Integration tests for startup validation.
//!
//! The empty-password-hash guard lives in [`lib_rpc::Server::run`]; these
//! tests verify it fires correctly before any service begins accepting requests.

const VALID_HASH: &str = "$argon2id$v=19$m=4096,t=3,p=1$Mzk3NGQ1NDBiZmZiYjhhNDY0YWY3MmRjYjIwNDI2YmE\
     $Bzo73djW0DkCVR4prXhqvWB4ViEdJ54h91mU0LCgqPY";

const TEST_TOKEN_SECRET: &str = "test_secret";

fn valid_access_token() -> String {
    lib_auth::AccessToken::new(&secrecy::SecretString::from(TEST_TOKEN_SECRET))
        .expect("access token must generate")
        .to_string()
}

fn rpc_settings(password_hash: &str) -> lib_settings::Settings {
    lib_settings::Settings {
        rpc: lib_settings::RpcSettings {
            host: "127.0.0.1".to_string(),
            port: 0,
            password_hash: password_hash.to_string(),
            token_secret: "test_secret".to_string(),
            ..Default::default()
        },
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
        lib_rpc::services::utilities::UtilitiesServiceClient::connect(format!("http://{addr}"))
            .await
            .unwrap();
    let mut request = tonic::Request::new(lib_rpc::services::utilities::PingRequest {});
    request
        .metadata_mut()
        .insert("access_token", valid_access_token().parse().unwrap());
    let response = client.ping(request).await.unwrap();

    assert_eq!(response.into_inner().pong, "Pong...");
}
