//! BitNode Console web-frontend-only server — serves the HTTP frontend without the gRPC backend.

mod error;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Run the BitNode Console web-frontend-only server.
pub async fn run() -> Result<()> {
    let settings: lib_settings::Settings = lib_settings::Settings::parse()?;

    if settings.tracing.enabled {
        let tracing_level = Some(settings.tracing.level);
        lib_tracing::init(tracing_level)?;
        if settings.tracing.show_settings_startup {
            tracing::info!("Starting web frontend server with settings: {:#?}", settings);
        }
    }

    let frontend_server =
        lib_web::HttpServer::new(&settings.frontend.host, settings.frontend.port);
    frontend_server.run().await.map_err(Error::from)?;

    Ok(())
}
