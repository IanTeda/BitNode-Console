//! Integration tests for the Journals gRPC endpoint.

use lib_rpc::services::journals::{GetJournalsRequest, StreamJournalsRequest};

use crate::support::TestRpcServer;

fn authenticated_get_journals() -> tonic::Request<GetJournalsRequest> {
    let mut request = tonic::Request::new(GetJournalsRequest::default());
    request.metadata_mut().insert(
        "access_token",
        crate::support::valid_access_token().parse().unwrap(),
    );
    request
}

fn authenticated_stream_journals(tail_lines: u32) -> tonic::Request<StreamJournalsRequest> {
    let mut request = tonic::Request::new(StreamJournalsRequest { tail_lines });
    request.metadata_mut().insert(
        "access_token",
        crate::support::valid_access_token().parse().unwrap(),
    );
    request
}

// --- get_journals: auth ---

#[tokio::test]
async fn get_journals_without_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.get_journals(tonic::Request::new(GetJournalsRequest::default())).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn get_journals_with_invalid_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let mut request = tonic::Request::new(GetJournalsRequest::default());
    request.metadata_mut().insert("access_token", "not.a.real.jwt".parse().unwrap());

    let result = client.get_journals(request).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

// --- get_journals: stub behaviour ---

#[tokio::test]
async fn get_journals_with_valid_token_returns_unimplemented() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.get_journals(authenticated_get_journals()).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unimplemented);
}

#[tokio::test]
async fn get_journals_default_request_is_accepted_by_server() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    // Zero-valued fields (no timestamp bounds, default priority, no pagination)
    // must reach the handler — the server should not reject the request shape.
    let status = client
        .get_journals(authenticated_get_journals())
        .await
        .unwrap_err();

    assert_ne!(status.code(), tonic::Code::InvalidArgument);
}

// --- stream_journals: auth ---

#[tokio::test]
async fn stream_journals_without_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client
        .stream_journals(tonic::Request::new(StreamJournalsRequest { tail_lines: 0 }))
        .await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn stream_journals_with_invalid_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let mut request = tonic::Request::new(StreamJournalsRequest { tail_lines: 0 });
    request.metadata_mut().insert("access_token", "not.a.real.jwt".parse().unwrap());

    let result = client.stream_journals(request).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

// --- stream_journals: stub behaviour ---

#[tokio::test]
async fn stream_journals_with_valid_token_returns_unimplemented() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.stream_journals(authenticated_stream_journals(0)).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unimplemented);
}

#[tokio::test]
async fn stream_journals_with_nonzero_tail_lines_returns_unimplemented() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.stream_journals(authenticated_stream_journals(50)).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unimplemented);
}
