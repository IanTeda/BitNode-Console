//! BitNode Console full server — serves both the web frontend and gRPC backend.

mod error;

//-- Re-exports to flatten the code structure
//
// Re-exports he error types for use in the application.
pub use error::Error;

// Re-exports the result type for use in the application.
pub type Result<T> = std::result::Result<T, Error>;

/// Run the BitNode Console backend application.
pub async fn run() -> Result<()> {
    //--- Get application settings
    let settings: lib_settings::Settings = lib_settings::Settings::parse()?;

    //--- Initialize telemetry if enabled
    if settings.tracing.enabled {
        let tracing_level = Some(settings.tracing.level);
        lib_tracing::init(tracing_level)?;
        if (settings.tracing.show_settings_startup) {
            tracing::info!("Starting server with settings: {:#?}", settings);
        }
    }

    //--- Create web server for serving HTTP ReactJS requests
    let frontend_server = lib_web::HttpServer::new(&settings.frontend.host, settings.frontend.port);
    let frontend_future = async { frontend_server.run().await.map_err(Error::from) };

    //--- Create RPC server for serving gRPC requests
    let backend_server = lib_rpc::Server::new(settings).await?;
    let backend_future = async { backend_server.run().await.map_err(Error::from) };

    //--- Join the web and RPC server futures to run them concurrently
    tokio::try_join!(frontend_future, backend_future)?;

    Ok(())
}
