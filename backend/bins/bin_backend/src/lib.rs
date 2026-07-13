//! BitNode Console RPC-only server — serves the gRPC backend without the web frontend.

mod error;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Run the BitNode Console RPC-only server.
pub async fn run() -> Result<()> {
    let settings: lib_settings::Settings = lib_settings::Settings::parse()?;

    if settings.tracing.enabled {
        let tracing_level = Some(settings.tracing.level);
        lib_tracing::init(tracing_level)?;
        if settings.tracing.show_settings_startup {
            tracing::info!("Starting RPC server with settings: {:#?}", settings);
        }
    }

    let rpc_server = lib_rpc::Server::new(settings).await?;
    rpc_server.run().await.map_err(Error::from)?;

    Ok(())
}
