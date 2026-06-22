//! Integration tests for the HTTP web server.

mod support;

use support::TestWebServer;

#[tokio::test]
async fn web_server_serves_index_html() {
    let server = TestWebServer::spawn().await;

    let response = server.get("/").await;

    assert_eq!(response.status(), 200);

    let body = response.text().await.unwrap();
    assert!(body.contains("html"), "expected HTML content in response body");
}

#[tokio::test]
async fn web_server_serves_fallback_for_unknown_routes() {
    let server = TestWebServer::spawn().await;

    let response = server.get("/nonexistent/path").await;

    let body = response.text().await.unwrap();
    assert!(
        body.contains("html"),
        "fallback should serve index.html content for unknown routes"
    );
}

#[tokio::test]
async fn web_server_bind_failure_returns_error() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    // Keep listener alive so the port is taken.

    let server = lib_web::HttpServer::new("127.0.0.1", port);
    let result = server.run().await;

    assert!(result.is_err(), "binding to a taken port should fail");
}
