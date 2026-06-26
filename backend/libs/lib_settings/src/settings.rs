//-- ./backend/libs/lib_settings/src/settings.rs

//! Settings struct and parsing logic for application configuration.

use config::Config;
use directories as Directories;
use std::path::{Path, PathBuf};

// use crate::{ApplicationSettings, Error, Result, TracingSettings, WebSettings};

/// Application name used for configuration directories and environment variables.
/// This should match the binary name and be used consistently across the application.
const APPLICATION_NAME: &str = "bitnode_console";

/// Environment variable prefix derived from the application name.
/// Converts "bitnode-console" to "`BITNODE_CONSOLE`" for environment variables.
const ENV_PREFIX: &str = "BITNODE_CONSOLE";

/// Settings for the Application.
///
/// This struct holds the parsed settings for the application, including
/// server, tracing and web configurations.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    #[serde(default)]
    pub application: crate::ApplicationSettings,

    #[serde(default)]
    pub rpc: crate::RpcSettings,

    #[serde(default)]
    pub tracing: crate::TracingSettings,

    #[serde(default)]
    pub web: crate::WebSettings,
}

impl Settings {
    /// Parses the configuration files from the various directories and environment variables.
    ///
    /// The function first applies default settings, then overrides them with higher precedence sources:
    /// 01. Built-in default config values (lowest)
    /// 02. System config directory
    /// 03. User config directory
    /// 04. Executable directory
    /// 05. Working directory
    /// 06. Explicit config file
    /// 07. Environment variables
    /// 08. Command line arguments (highest)
    /// 09. Build the config
    ///
    /// # Errors
    ///
    /// Returns an error if the current directory or executable path cannot be
    /// determined, if any configuration source fails to parse, or if the
    /// merged configuration cannot be deserialized into `Settings<S>`.
    pub fn parse(config_file: Option<&Path>) -> crate::Result<Self> {
        //--- 01. Build-in defaults
        // Seed the config builder with the default configuration so that
        // any fields not supplied by later sources default back to this.
        let defaults = Self::default();
        let mut config_builder =
            Config::builder().add_source(Config::try_from(&defaults).map_err(|err| {
                let msg = format!("Error parsing default settings: {err}");
                crate::Error::Parsing(msg)
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
        // The config file path passed into the parse method. Typically the --config --c CLI argument.

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
            config_builder.build().map_err(|err| crate::Error::Generic(err.to_string()))?;

        let settings: Self =
            config.try_deserialize().map_err(|err| crate::Error::Generic(err.to_string()))?;

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

    #[test]
    fn default_applies_all_section_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.application.password(), "");
        assert!(settings.tracing.enabled);
        assert_eq!(settings.tracing.level, lib_tracing::Levels::INFO);
        assert!(!settings.tracing.show_settings_startup);
        assert_eq!(settings.web.port, 8090);
        assert_eq!(settings.web.host, "127.0.0.1");
    }

    #[test]
    fn clone_produces_equal_fields() {
        let original = Settings::default();
        let cloned = original.clone();
        assert_eq!(original.application, cloned.application);
        assert_eq!(original.tracing, cloned.tracing);
        assert_eq!(original.web.port, cloned.web.port);
        assert_eq!(original.web.host, cloned.web.host);
    }

    #[test]
    fn debug_format_includes_struct_name() {
        let settings = Settings::default();
        let debug_str = format!("{settings:?}");
        assert!(debug_str.contains("Settings"));
    }

    #[test]
    fn deserialize_from_empty_json_uses_section_defaults() {
        let settings: Settings = serde_json::from_str("{}").expect("deserialize Settings");
        assert_eq!(settings.web.port, 8090);
        assert_eq!(settings.tracing.level, lib_tracing::Levels::INFO);
        assert_eq!(settings.application.password(), "");
    }

    #[test]
    fn deserialize_partial_section_still_requires_all_section_fields() {
        // [web] section without `host` — serde requires all WebSettings fields
        let json = r#"{"web": {"port": 9000}}"#;
        let result: Result<Settings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_deserialize_json_roundtrip() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).expect("serialize Settings");
        let deserialized: Settings = serde_json::from_str(&json).expect("deserialize Settings");
        assert_eq!(deserialized.web.port, settings.web.port);
        assert_eq!(deserialized.web.host, settings.web.host);
        assert_eq!(deserialized.tracing, settings.tracing);
        assert_eq!(deserialized.application, settings.application);
    }

    #[test]
    fn parse_with_no_config_file_returns_defaults() {
        let settings = Settings::parse(None).expect("parse should succeed");
        assert_eq!(settings.web.port, 8090);
        assert_eq!(settings.tracing.level, lib_tracing::Levels::INFO);
        assert_eq!(settings.application.password(), "");
    }

    #[test]
    fn parse_overrides_web_section_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[web]").unwrap();
        writeln!(file, "port = 9100").unwrap();
        writeln!(file, "host = 0.0.0.0").unwrap();

        let settings = Settings::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(settings.web.port, 9100);
        assert_eq!(settings.web.host, "0.0.0.0");
    }

    #[test]
    fn parse_overrides_tracing_level_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = debug").unwrap();

        let settings = Settings::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(settings.tracing.level, lib_tracing::Levels::DEBUG);
    }

    #[test]
    fn parse_overrides_tracing_enabled_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "enabled = false").unwrap();

        let settings = Settings::parse(Some(file.path())).expect("parse should succeed");

        assert!(!settings.tracing.enabled);
    }

    #[test]
    fn parse_overrides_application_section_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[application]").unwrap();
        writeln!(file, "password = hunter2").unwrap();

        let settings = Settings::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(settings.application.password(), "hunter2");
    }

    #[test]
    fn parse_applies_multiple_sections_from_one_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[web]").unwrap();
        writeln!(file, "port = 9200").unwrap();
        writeln!(file, "host = 0.0.0.0").unwrap();
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = warn").unwrap();

        let settings = Settings::parse(Some(file.path())).expect("parse should succeed");

        assert_eq!(settings.web.port, 9200);
        assert_eq!(settings.tracing.level, lib_tracing::Levels::WARN);
    }

    #[test]
    fn parse_with_missing_config_file_fails() {
        let result = Settings::parse(Some(Path::new("/nonexistent/bitnode_console.conf")));
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_invalid_port_value_fails() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[web]").unwrap();
        writeln!(file, "port = not_a_number").unwrap();

        let result = Settings::parse(Some(file.path()));
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_invalid_tracing_level_fails() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = verbose").unwrap();

        let result = Settings::parse(Some(file.path()));
        assert!(result.is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn get_system_config_path_returns_etc_path_on_linux() {
        let path = Settings::get_system_config_path()
            .expect("system config path should be defined on linux");
        assert_eq!(
            path,
            PathBuf::from("/etc/bitnode_console/bitnode_console.conf")
        );
    }
}
