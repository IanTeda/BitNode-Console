//! Settings Library Module
//!
//! This library is used to load and validate configuration files.

mod error;

use config::Config;
use directories;
use std::path::Path;

/// Settings loading and validation errors.
pub use error::{SettingsError, SettingsResult};

/// Application name used for configuration directories and environment variables.
/// This should match the binary name and be used consistently across the application.
///
/// # Changing the Application Name
///
/// To use this configuration system for a different application:
/// 1. Change this constant to match your application name
/// 2. Update the corresponding ENV_PREFIX if needed
/// 3. Ensure your binary name matches this constant
const APPLICATION_NAME: &str = "bitnode_console";

/// Environment variable prefix derived from the application name.
/// Converts "bitnode-console" to "BITNODE_CONSOLE" for environment variables.
///
/// # Changing the Environment Prefix
///
/// If your application name contains characters that aren't valid in environment
/// variable names, update this constant accordingly.
const ENV_PREFIX: &str = "BITNODE_CONSOLE";

/// Configuration for the API.
///
/// The `server` field is generic so that each binary crate (e.g. `server`)
/// can supply its own server-specific settings type, avoiding a dependency
/// from this crate back onto its consumers.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct Settings<S> {
    pub server: S,
}

impl<S> Settings<S>
where
    S: Default + serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn parse(config_file: Option<&Path>) -> SettingsResult<Settings<S>> {
        // Higher precedence sources override lower precedence ones:
        // 01. Built-in default config values (lowest)
        // 02. System config directory
        // 03. User config directory
        // 04. Executable directory
        // 05. Working directory
        // 06. Explicit config file
        // 07. Environment variables
        // 08. Command line arguments (highest)
        // 09. Build the config

        //--- 01. Build-in defaults
        // Seed the config builder with the default configuration so that
        // any fields not supplied by later sources default back to this.
        let defaults = Settings::<S>::default();
        let mut config_builder =
            Config::builder().add_source(Config::try_from(&defaults).map_err(|err| {
                let msg = format!("Error parsing default settings: {err}");
                SettingsError::Parsing(msg)
            })?);

        //--- 02. System config directory
        // The system-wide config directory, typically

        // TODO: Add system config directory source

        //--- 03. User config directory
        // The user-specific config directory, typically ~/.local/share/ or ~/.config/

        // TODO: Add user config directory source

        //--- 04. Executable directory
        // The directory where the actual executable file physically lives on disk.

        // TODO: Add executable directory source

        //--- 05. Working directory
        // The directory you are "in" when you run the program (the folder your shell is currently in).

        let working_directory_path =
            std::env::current_dir()?.join(format!("{APPLICATION_NAME}.conf"));

        if working_directory_path.exists() {
            config_builder = config_builder.add_source(
                config::File::from(working_directory_path).format(config::FileFormat::Ini),
            );
        }

        //--- 06. Explicit config file
        // The config path passed into the `Settings::parse` method. Typically the --config --c CLI argument.

        //--- 07. Environment variables
        // Environment variables read from the process environment.

        //--- 08. Command line arguments
        // TODO: Add command line arguments source

        //--- 09. Build the config
        let config =
            config_builder.build().map_err(|err| SettingsError::Generic(err.to_string()))?;

        config.try_deserialize().map_err(|err| SettingsError::Generic(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal stand-in for a consumer-supplied server settings type, used
    /// to exercise `Settings<S>` without depending on the `server` crate.
    #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
    struct TestServerSettings {
        port: u16,
        host: String,
    }

    #[test]
    fn test_default() {
        let settings = Settings::<TestServerSettings>::default();
        assert_eq!(settings.server, TestServerSettings::default());
    }

    #[test]
    fn test_parse_returns_defaults() {
        let settings = Settings::<TestServerSettings>::parse(None).expect("parse should succeed");

        assert_eq!(settings.server, TestServerSettings::default());
    }

    #[test]
    fn test_clone() {
        let settings = Settings::<TestServerSettings> {
            server: TestServerSettings {
                port: 9000,
                host: "0.0.0.0".to_string(),
            },
        };

        let cloned = settings.clone();

        assert_eq!(settings.server, cloned.server);
    }

    #[test]
    fn test_debug_format() {
        let settings = Settings::<TestServerSettings>::default();

        let debug_str = format!("{settings:?}");

        assert!(debug_str.contains("Settings"));
        assert!(debug_str.contains("TestServerSettings"));
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{"server": {"port": 9000, "host": "0.0.0.0"}}"#;

        let settings: Settings<TestServerSettings> =
            serde_json::from_str(json).expect("deserialize Settings");

        assert_eq!(settings.server.port, 9000);
        assert_eq!(settings.server.host, "0.0.0.0");
    }

    #[test]
    fn test_deserialize_missing_server_fails() {
        let json = "{}";

        let result: Result<Settings<TestServerSettings>, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }
}
