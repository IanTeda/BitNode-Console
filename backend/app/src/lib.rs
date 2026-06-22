//! BitNode Console backend application library.

mod error;

//-- Re-exports to flatten the code structure

// Re-exports he error types for use in the application.
pub use error::Error;

// Re-exports the result type for use in the application.
pub type Result<T> = std::result::Result<T, Error>;

/// Run the BitNode Console backend application.
pub async fn run() -> Result<()> {
    let settings: lib_settings::Settings = lib_settings::Settings::parse(None)?;

    if settings.tracing.enabled {
        let tracing_level = Some(settings.tracing.level);
        lib_tracing::init(tracing_level)?;
        if (settings.tracing.show_settings_startup) {
            tracing::info!("Starting server with settings: {:#?}", settings);
        }
    }

    let web_server = lib_web::HttpServer::new(&settings.web.host, settings.web.port);
    let web_future = async { web_server.run().await.map_err(Error::from) };

    let rpc_address = settings.rpc.socket_address()?;
    let rpc_server = lib_rpc::Server::new(rpc_address).await?;
    let rpc_future = async { rpc_server.run().await.map_err(Error::from) };

    tokio::try_join!(web_future, rpc_future)?;

    Ok(())
}
