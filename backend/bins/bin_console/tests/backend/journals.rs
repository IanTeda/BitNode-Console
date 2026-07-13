//! Integration tests for the Journals gRPC endpoint.

use lib_rpc::services::journals::{FollowJournalsRequest, GetJournalsRequest};

use crate::support::TestRpcServer;

fn authenticated_get_journals() -> tonic::Request<GetJournalsRequest> {
    let mut request = tonic::Request::new(GetJournalsRequest::default());
    request.metadata_mut().insert(
        "access_token",
        crate::support::valid_access_token().parse().unwrap(),
    );
    request
}

fn authenticated_follow_journals(tail_lines: Option<u32>) -> tonic::Request<FollowJournalsRequest> {
    let mut request = tonic::Request::new(FollowJournalsRequest {
        tail_lines,
        priority: None,
    });
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

// --- get_journals: behaviour ---

#[tokio::test]
async fn get_journals_with_valid_token_returns_ok() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.get_journals(authenticated_get_journals()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn get_journals_default_request_is_accepted_by_server() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    // Zero-valued fields (no timestamp bounds, default priority, no pagination)
    // must reach the handler — the server should not reject the request shape.
    let result = client.get_journals(authenticated_get_journals()).await;

    match result {
        Ok(_) => {},
        Err(status) => assert_ne!(status.code(), tonic::Code::InvalidArgument),
    }
}

// --- follow_journals: auth ---

#[tokio::test]
async fn follow_journals_without_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client
        .follow_journals(tonic::Request::new(FollowJournalsRequest {
            tail_lines: None,
            priority: None,
        }))
        .await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn follow_journals_with_invalid_token_returns_unauthenticated() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let mut request = tonic::Request::new(FollowJournalsRequest {
        tail_lines: None,
        priority: None,
    });
    request.metadata_mut().insert("access_token", "not.a.real.jwt".parse().unwrap());

    let result = client.follow_journals(request).await;

    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

// --- follow_journals: behaviour ---

#[tokio::test]
async fn follow_journals_with_valid_token_returns_stream() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.follow_journals(authenticated_follow_journals(None)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn follow_journals_with_nonzero_tail_lines_returns_stream() {
    let app = TestRpcServer::spawn().await;
    let mut client = app.journals_client().await;

    let result = client.follow_journals(authenticated_follow_journals(Some(50))).await;

    assert!(result.is_ok());
}
