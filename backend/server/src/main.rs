//! Main entry point for the BitNode Console backend server.

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::{error::ServerError, prelude::*, settings::Settings};

mod error;
mod prelude;
mod settings;

const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

#[tokio::main]
async fn main() -> ServerResult<()> {
    let settings = Settings::parse(None)?;

    println!("Server settings: {:?}", settings);

    let index_html = format!("{ASSETS_DIR}/index.html");
    let static_files = ServeDir::new(ASSETS_DIR).not_found_service(ServeFile::new(index_html));

    // Fall back to index.html for client-side (SPA) routing.
    let app = Router::new().fallback_service(static_files);

    // run our app with hyper, listening on the configured server address
    let listener = tokio::net::TcpListener::bind(settings.server.address()).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
