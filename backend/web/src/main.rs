use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

#[tokio::main]
async fn main() {
    let index_html = format!("{ASSETS_DIR}/index.html");
    let static_files = ServeDir::new(ASSETS_DIR).not_found_service(ServeFile::new(index_html));

    // Fall back to index.html for client-side (SPA) routing.
    let app = Router::new().fallback_service(static_files);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
