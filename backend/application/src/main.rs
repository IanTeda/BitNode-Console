//! Main entry point for the BitNode Console backend server.

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

mod error;
mod prelude;

use crate::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse settings from the config file.
    let settings: lib_settings::Settings = lib_settings::Settings::parse(None)?;

    if settings.tracing.enabled {
        let tracing_level = Some(settings.tracing.level);
        lib_tracing::init(tracing_level)?;
        if (settings.tracing.show_settings_startup) {
            tracing::info!("Starting server with settings: {:#?}", settings);
        }
    }

    let frontend_web_server = lib_web::HttpServer::new(&settings.web.host, settings.web.port);
    frontend_web_server.run().await?;

    Ok(())
}
