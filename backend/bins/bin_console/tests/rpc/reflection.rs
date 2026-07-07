//! Integration tests confirming gRPC server reflection is active and complete.

use tokio_stream::StreamExt;
use tonic_reflection::pb::v1::{
    server_reflection_client::ServerReflectionClient,
    server_reflection_request::MessageRequest,
    server_reflection_response::MessageResponse,
    ServerReflectionRequest,
};

use crate::support::TestRpcServer;

/// Send a single reflection request to `app` and return the message response.
async fn reflect(app: &TestRpcServer, req: MessageRequest) -> MessageResponse {
    let conn = tonic::transport::Endpoint::new(format!("http://{}", app.address))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = ServerReflectionClient::new(conn);

    let mut inbound = client
        .server_reflection_info(tonic::Request::new(tokio_stream::once(
            ServerReflectionRequest {
                host: String::new(),
                message_request: Some(req),
            },
        )))
        .await
        .unwrap()
        .into_inner();

    inbound
        .next()
        .await
        .expect("stream ended before response")
        .expect("response error")
        .message_response
        .expect("missing message_response")
}

// --- list services ---

#[tokio::test]
async fn reflection_lists_authentication_service() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(&app, MessageRequest::ListServices(String::new())).await;

    let MessageResponse::ListServicesResponse(services) = response else {
        panic!("expected ListServicesResponse");
    };
    assert!(
        services
            .service
            .iter()
            .any(|s| s.name == "bitnode_console.authentication.v1.AuthenticationService"),
        "AuthenticationService missing from service list: {:?}",
        services.service
    );
}

#[tokio::test]
async fn reflection_lists_utilities_service() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(&app, MessageRequest::ListServices(String::new())).await;

    let MessageResponse::ListServicesResponse(services) = response else {
        panic!("expected ListServicesResponse");
    };
    assert!(
        services
            .service
            .iter()
            .any(|s| s.name == "bitnode_console.utilities.v1.UtilitiesService"),
        "UtilitiesService missing from service list: {:?}",
        services.service
    );
}

#[tokio::test]
async fn reflection_lists_journals_service() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(&app, MessageRequest::ListServices(String::new())).await;

    let MessageResponse::ListServicesResponse(services) = response else {
        panic!("expected ListServicesResponse");
    };
    assert!(
        services
            .service
            .iter()
            .any(|s| s.name == "bitnode_console.journals.v1.JournalsService"),
        "JournalsService missing from service list: {:?}",
        services.service
    );
}

// --- symbol resolution ---

#[tokio::test]
async fn reflection_resolves_authentication_service_symbol() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(
        &app,
        MessageRequest::FileContainingSymbol(
            "bitnode_console.authentication.v1.AuthenticationService".to_string(),
        ),
    )
    .await;

    assert!(
        matches!(response, MessageResponse::FileDescriptorResponse(_)),
        "expected FileDescriptorResponse, got: {response:?}"
    );
}

#[tokio::test]
async fn reflection_resolves_utilities_service_symbol() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(
        &app,
        MessageRequest::FileContainingSymbol(
            "bitnode_console.utilities.v1.UtilitiesService".to_string(),
        ),
    )
    .await;

    assert!(
        matches!(response, MessageResponse::FileDescriptorResponse(_)),
        "expected FileDescriptorResponse, got: {response:?}"
    );
}

#[tokio::test]
async fn reflection_resolves_journals_service_symbol() {
    let app = TestRpcServer::spawn().await;
    let response = reflect(
        &app,
        MessageRequest::FileContainingSymbol(
            "bitnode_console.journals.v1.JournalsService".to_string(),
        ),
    )
    .await;

    assert!(
        matches!(response, MessageResponse::FileDescriptorResponse(_)),
        "expected FileDescriptorResponse, got: {response:?}"
    );
}
