//! Integration tests for the gRPC RPC server.

use crate::support::TestRpcServer;

#[tokio::test]
async fn rpc_server_starts_and_responds_to_ping() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.rpc_client().await;

    let response = client.ping(tonic::Request::new(lib_rpc::PingRequest {})).await.unwrap();

    assert_eq!(response.into_inner().pong, "Pong...");
}

#[tokio::test]
async fn rpc_server_handles_multiple_sequential_pings() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.rpc_client().await;

    for _ in 0..5 {
        let response = client.ping(tonic::Request::new(lib_rpc::PingRequest {})).await.unwrap();

        assert_eq!(response.into_inner().pong, "Pong...");
    }
}

#[tokio::test]
async fn rpc_server_handles_concurrent_clients() {
    let app = TestRpcServer::spawn().await;
    let addr = app.address;

    let mut handles = Vec::new();

    for _ in 0..5 {
        handles.push(tokio::spawn(async move {
            let mut client = lib_rpc::UtilitiesServiceClient::connect(format!("http://{addr}"))
                .await
                .unwrap();

            let response = client.ping(tonic::Request::new(lib_rpc::PingRequest {})).await.unwrap();

            assert_eq!(response.into_inner().pong, "Pong...");
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn rpc_server_bind_failure_returns_error() {
    let first = lib_rpc::Server::new(crate::support::test_settings()).await.unwrap();
    let taken_port = first.address().unwrap().port();

    let conflict_settings = lib_settings::RpcSettings {
        port: taken_port,
        ..crate::support::test_settings()
    };
    let result = lib_rpc::Server::new(conflict_settings).await;

    assert!(result.is_err(), "binding to a taken port should fail");
}

#[tokio::test]
async fn rpc_client_fails_to_connect_to_unbound_port() {
    let result = lib_rpc::UtilitiesServiceClient::connect("http://127.0.0.1:1").await;

    assert!(result.is_err());
}
