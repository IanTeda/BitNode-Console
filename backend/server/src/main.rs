//! Main entry point for the BitNode Console backend server.

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

mod error;
mod prelude;
mod settings;

use crate::{error::ServerError, prelude::*, settings::Settings};

const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

#[tokio::main]
async fn main() -> ServerResult<()> {
    // Parse settings from the config file.
    let settings = Settings::parse(None)?;

    let telemetry_level = Some(settings.telemetry.telemetry_level());
    lib_tracing::init(telemetry_level)?;
    tracing::info!("Starting server with settings: {:#?}", settings);

    let index_html = format!("{ASSETS_DIR}/index.html");
    let static_files = ServeDir::new(ASSETS_DIR).not_found_service(ServeFile::new(index_html));

    // Fall back to index.html for client-side (SPA) routing.
    let app = Router::new().fallback_service(static_files);

    // run our app with hyper, listening on the configured server address
    let address = settings.server.address();
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    println!("Listening on {address}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
