//-- ./backend/libs/lib_settings/src/lib.rs

//! Settings Library Module
//!
//! This library is used to load and validate configuration files.

mod application;
mod bitcoind;
mod cli;
mod error;
mod backend;
mod settings;
mod tracing;
mod web;

/// Re-export Settings error type
pub use error::Error;

/// Re-export CLI argument struct.
pub use cli::Cli;

/// Result type alias used across the settings module.
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export settings types.
pub use application::ApplicationSettings;
pub use bitcoind::BitcoinDaemonSettings;
pub use backend::BackendSettings;
pub use settings::Settings;
pub use tracing::TracingSettings;
pub use web::WebSettings;
