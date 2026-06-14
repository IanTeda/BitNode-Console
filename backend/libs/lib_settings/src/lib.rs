//! Settings Library Module
//!
//! This library is used to load and validate configuration files.

mod error;

use config::Config;
use std::path::Path;

/// Settings loading and validation errors.
pub use error::{SettingsError, SettingsResult};

/// Configuration for the API.
///
/// The `server` field is generic so that each binary crate (e.g. `server`)
/// can supply its own server-specific settings type, avoiding a dependency
/// from this crate back onto its consumers.
#[derive(serde::Deserialize, Clone, Debug, Default)]
pub struct Settings<S> {
    pub server: S,
}

impl<S> Settings<S>
where
    S: Default + serde::de::DeserializeOwned,
{
    pub fn parse(config_file: Option<&Path>) -> SettingsResult<Settings<S>> {
        // Higher precedence sources override lower precedence ones:
        // 1. Built-in default config values (lowest)
        // 2. System config directory
        // 3. User config directory
        // 4. Executable directory
        // 5. Working directory
        // 6. Explicit config file
        // 7. Environment variables
        // 8. Command line arguments (highest)

        //--- 01. Build-in defaults
        // Build the default configuration using the `Default` trait
        let mut settings = Settings::<S>::default();

        //--- 02. System config directory

        //--- 03. User config directory

        //--- 04. Executable directory

        //--- 05. Working directory

        //--- 06. Explicit config file

        //--- 07. Environment variables

        Ok(settings)
    }
}
