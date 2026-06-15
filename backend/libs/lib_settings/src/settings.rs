//-- ./backend/libs/lib_settings/src/settings.rs

//! Settings struct and configuration parsing logic.

use config::Config;
use directories as Directories;
use std::path::{Path, PathBuf};

use crate::{SettingsError, SettingsResult};

/// Application name used for configuration directories and environment variables.
/// This should match the binary name and be used consistently across the application.
///
/// # Changing the Application Name
///
/// To use this configuration system for a different application:
/// 1. Change this constant to match your application name
/// 2. Update the corresponding `ENV_PREFIX` if needed
/// 3. Ensure your binary name matches this constant
const APPLICATION_NAME: &str = "bitnode_console";

/// Environment variable prefix derived from the application name.
/// Converts "bitnode-console" to "`BITNODE_CONSOLE`" for environment variables.
///
/// # Changing the Environment Prefix
///
/// If your application name contains characters that aren't valid in environment
/// variable names, update this constant accordingly.
const ENV_PREFIX: &str = "BITNODE_CONSOLE";

/// Settings for the Application.
///
/// The `server` field is generic so that each binary crate (e.g. `server`)
/// can supply its own server-specific settings type, avoiding a dependency
/// from this crate back onto its consumers.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct Settings<S> {
    pub server: S,

    /// Telemetry (logging and tracing) configuration.
    #[serde(default)]
    pub telemetry: lib_telemetry::TelemetrySettings,
}

impl<S> Settings<S>
where
    S: Default + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Parses the configuration files from the various directories and environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the current directory or executable path cannot be
    /// determined, if any configuration source fails to parse, or if the
    /// merged configuration cannot be deserialized into `Settings<S>`.
    pub fn parse(config_file: Option<&Path>) -> SettingsResult<Self> {
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
        let defaults = Self::default();
        let mut config_builder =
            Config::builder().add_source(Config::try_from(&defaults).map_err(|err| {
                let msg = format!("Error parsing default settings: {err}");
                SettingsError::Parsing(msg)
            })?);

        //--- 02. System config directory
        // The system-wide config directory, typically /etc/.

        if let Some(system_config_file) = Self::get_system_config_path().filter(|p| p.exists()) {
            config_builder = config_builder
                .add_source(config::File::from(system_config_file).format(config::FileFormat::Ini));
        }

        //--- 03. User config directory
        // The user-specific config directory, typically ~/.local/share/ or ~/.config/

        if let Some(project_dirs) =
            Directories::ProjectDirs::from("au.id", "teda", APPLICATION_NAME)
        {
            let user_config_file =
                project_dirs.config_dir().join(format!("{APPLICATION_NAME}.conf"));

            if user_config_file.exists() {
                config_builder = config_builder.add_source(
                    config::File::from(user_config_file).format(config::FileFormat::Ini),
                );
            }
        }

        //--- 04. Executable directory
        // The directory where the actual executable file physically lives on disk.

        if let Some(executable_directory) =
            std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))
        {
            let executable_config_file =
                executable_directory.join(format!("{APPLICATION_NAME}.conf"));

            if executable_config_file.exists() {
                config_builder = config_builder.add_source(
                    config::File::from(executable_config_file).format(config::FileFormat::Ini),
                );
            }
        }

        //--- 05. Working directory
        // The directory you are "in" when you run the program (the folder your shell is currently in).

        let working_config_file = std::env::current_dir()?.join(format!("{APPLICATION_NAME}.conf"));

        if working_config_file.exists() {
            config_builder = config_builder.add_source(
                config::File::from(working_config_file).format(config::FileFormat::Ini),
            );
        }

        //--- 06. Explicit config file
        // The config path passed into the parse method. Typically the --config --c CLI argument.

        if let Some(explicit_config_file) = config_file {
            config_builder = config_builder.add_source(
                config::File::from(explicit_config_file.to_path_buf())
                    .format(config::FileFormat::Ini),
            );
        }

        //--- 07. Environment variables
        // Environment variables with the prefix `<ENV_PREFIX>` read from the process environment.

        config_builder = config_builder.add_source(config::Environment::with_prefix(ENV_PREFIX));

        //--- 08. Command line arguments
        // Override all other config values with command line arguments passed to the application binary.

        // TODO: Add command line arguments source using CLAP

        //--- 09. Build the config
        // Take the config builder, build the config, and deserialize it into a `Settings` struct. Then
        // return the `Settings` struct.

        let config =
            config_builder.build().map_err(|err| SettingsError::Generic(err.to_string()))?;

        let settings: Self = config
            .try_deserialize()
            .map_err(|err| SettingsError::Generic(err.to_string()))?;

        Ok(settings)
    }

    /// Get the system-wide configuration file path.
    ///
    /// Returns the path to the system configuration file using platform-specific
    /// standard locations. This provides system administrators with a way to
    /// set default configurations for all users.
    ///
    /// - **Unix/Linux**: `/etc/<APPLICATION_NAME>/<APPLICATION_NAME>.conf`
    /// - **Windows**: `%ALLUSERSPROFILE%\<APPLICATION_NAME>\<APPLICATION_NAME>.conf`
    /// - **macOS**: `/Library/Preferences/<APPLICATION_NAME>/<APPLICATION_NAME>.conf`
    /// - **Other**: None (system config not supported)
    // `Option` is unavoidable here: the Windows and "other" branches genuinely
    // return `None`, even though the Linux/macOS branches always return `Some`.
    #[allow(clippy::unnecessary_wraps)]
    fn get_system_config_path() -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            Some(
                PathBuf::from("/etc")
                    .join(APPLICATION_NAME)
                    .join(format!("{APPLICATION_NAME}.conf")),
            )
        }
        #[cfg(target_os = "macos")]
        {
            Some(
                PathBuf::from("/Library/Preferences")
                    .join(APPLICATION_NAME)
                    .join(format!("{APPLICATION_NAME}.conf")),
            )
        }
        #[cfg(target_os = "windows")]
        {
            // On Windows, use ALLUSERSPROFILE for system-wide settings
            std::env::var_os("ALLUSERSPROFILE").map(|all_users| {
                PathBuf::from(all_users)
                    .join(APPLICATION_NAME)
                    .join(format!("{APPLICATION_NAME}.conf"))
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
            telemetry: lib_telemetry::TelemetrySettings::default(),
        };

        let cloned = settings.clone();

        assert_eq!(settings.server, cloned.server);
        assert_eq!(settings.telemetry, cloned.telemetry);
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

    #[test]
    fn test_parse_with_explicit_config_file_overrides_defaults() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp config file");
        writeln!(file, "[server]").expect("write temp config file");
        writeln!(file, "port = 9100").expect("write temp config file");
        writeln!(file, "host = 127.0.0.1").expect("write temp config file");

        let settings =
            Settings::<TestServerSettings>::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(settings.server.port, 9100);
        assert_eq!(settings.server.host, "127.0.0.1");
    }

    #[test]
    fn test_parse_with_explicit_config_file_overrides_telemetry_level() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp config file");
        writeln!(file, "[telemetry]").expect("write temp config file");
        writeln!(file, "telemetry_level = debug").expect("write temp config file");

        let settings =
            Settings::<TestServerSettings>::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(
            settings.telemetry.telemetry_level,
            lib_telemetry::TelemetryLevels::DEBUG
        );
    }

    #[test]
    fn test_parse_with_invalid_explicit_config_file_fails() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp config file");
        writeln!(file, "[server]").expect("write temp config file");
        writeln!(file, "port = not_a_number").expect("write temp config file");

        let result = Settings::<TestServerSettings>::parse(Some(file.path()));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_missing_explicit_config_file_fails() {
        let missing_path = Path::new("/nonexistent/path/to/bitnode_console.conf");

        let result = Settings::<TestServerSettings>::parse(Some(missing_path));

        assert!(result.is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_system_config_path_linux() {
        let path = Settings::<TestServerSettings>::get_system_config_path()
            .expect("system config path should be defined on linux");

        assert_eq!(
            path,
            PathBuf::from("/etc/bitnode_console/bitnode_console.conf")
        );
    }
}
