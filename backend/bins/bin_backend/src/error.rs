//! Error types for the `bin_backend` binary.

/// Top-level error type for the RPC-only server.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic error {0}")]
    Generic(String),

    /// RPC server errors.
    #[error("RPC error: {0}")]
    Rpc(#[from] lib_rpc::Error),

    /// Settings loading or validating errors.
    #[error("Settings error: {0}")]
    Settings(#[from] lib_settings::Error),

    /// Telemetry initialisation errors.
    #[error("Tracing error: {0}")]
    Tracing(#[from] lib_tracing::Error),
}
