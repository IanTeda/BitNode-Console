//-- ./backend/libs/lib_settings/src/settings.rs

//! Settings struct and parsing logic for application configuration.

use config::Config;
use directories as Directories;
use std::path::{Path, PathBuf};

// use crate::{ApplicationSettings, Error, Result, TracingSettings, WebSettings};

/// Application name used for configuration directories and environment variables.
/// This should match the binary name and be used consistently across the application.
const APPLICATION_NAME: &str = "bitnode_console";

/// Environment variable prefix for all configuration overrides.
const ENV_PREFIX: &str = "BITNODE";

/// Settings for the Application.
///
/// This struct holds the parsed settings for the application, including
/// server, tracing and web configurations.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    #[serde(default)]
    pub application: crate::ApplicationSettings,

    #[serde(default)]
    pub bitcoind: crate::BitcoinDaemonSettings,

    #[serde(default)]
    pub rpc: crate::RpcSettings,

    #[serde(default)]
    pub tracing: crate::TracingSettings,

    #[serde(default)]
    pub web: crate::WebSettings,
}

impl Settings {
    /// Parses configuration from all sources, with CLI flags as the highest-priority override.
    ///
    /// Calls [`clap::Parser::parse`] to read process arguments, then delegates to
    /// [`Settings::parse_with_cli`]. Sources are applied in ascending priority:
    /// 01. Built-in defaults → 02. System config → 03. User config →
    /// 04. Executable dir → 05. Working dir → 06. Explicit config file (from `--config`) →
    /// 07. Environment variables → 08. CLI flags (highest).
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration source fails to parse or if the merged
    /// configuration cannot be deserialized into [`Settings`].
    pub fn parse() -> crate::Result<Self> {
        use clap::Parser as _;
        Self::parse_with_cli(&crate::Cli::parse())
    }

    /// Builds [`Settings`] from an explicit [`Cli`] value.
    ///
    /// This is the inner implementation used by [`Settings::parse`] and by tests
    /// that need to supply a specific config file path or flag values without
    /// going through process argument parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the current directory or executable path cannot be
    /// determined, if any configuration source fails to parse, or if the
    /// merged configuration cannot be deserialized into [`Settings`].
    pub(crate) fn parse_with_cli(cli: &crate::Cli) -> crate::Result<Self> {
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
        // The path supplied via the --config / -c CLI flag.

        if let Some(explicit_config_file) = &cli.config {
            config_builder = config_builder.add_source(
                config::File::from(explicit_config_file.clone()).format(config::FileFormat::Ini),
            );
        }

        //--- 07. Environment variables
        // Environment variables with the prefix `<ENV_PREFIX>` read from the process environment.

        config_builder = config_builder.add_source(config::Environment::with_prefix(ENV_PREFIX));

        //--- 08. Command line arguments (highest priority)
        // Individual flag values override every other source via set_override.

        if let Some(enabled) = cli.tracing_enabled {
            config_builder = config_builder
                .set_override("tracing.enabled", enabled)
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(level) = cli.tracing_level {
            config_builder = config_builder
                .set_override("tracing.level", level.to_string())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(show) = cli.tracing_show_settings_startup {
            config_builder = config_builder
                .set_override("tracing.show_settings_startup", show)
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(port) = cli.rpc_port {
            config_builder = config_builder
                .set_override("rpc.port", i64::from(port))
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref host) = cli.rpc_host {
            config_builder = config_builder
                .set_override("rpc.host", host.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref hash) = cli.rpc_password_hash {
            config_builder = config_builder
                .set_override("rpc.password_hash", hash.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref secret) = cli.rpc_token_secret {
            config_builder = config_builder
                .set_override("rpc.token_secret", secret.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref ips) = cli.rpc_allowed_ips {
            let ip_strings: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();
            config_builder = config_builder
                .set_override("rpc.allowed_ips", ip_strings)
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref name) = cli.bitcoind_unit_name {
            config_builder = config_builder
                .set_override("bitcoind.unit_name", name.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref host) = cli.bitcoind_rpc_host {
            config_builder = config_builder
                .set_override("bitcoind.rpc_host", host.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(port) = cli.bitcoind_rpc_port {
            config_builder = config_builder
                .set_override("bitcoind.rpc_port", i64::from(port))
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref user) = cli.bitcoind_rpc_user {
            config_builder = config_builder
                .set_override("bitcoind.rpc_user", user.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref password) = cli.bitcoind_rpc_password {
            config_builder = config_builder
                .set_override("bitcoind.rpc_password", password.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref path) = cli.bitcoind_cookie_file {
            let path_str = path.to_string_lossy();
            config_builder = config_builder
                .set_override("bitcoind.cookie_file", path_str.as_ref())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(port) = cli.web_port {
            config_builder = config_builder
                .set_override("web.port", i64::from(port))
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

        if let Some(ref host) = cli.web_host {
            config_builder = config_builder
                .set_override("web.host", host.as_str())
                .map_err(|err| crate::Error::Generic(err.to_string()))?;
        }

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
        let settings =
            Settings::parse_with_cli(&crate::Cli::default()).expect("parse should succeed");
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

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert_eq!(settings.web.port, 9100);
        assert_eq!(settings.web.host, "0.0.0.0");
    }

    #[test]
    fn parse_overrides_tracing_level_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = debug").unwrap();

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert_eq!(settings.tracing.level, lib_tracing::Levels::DEBUG);
    }

    #[test]
    fn parse_overrides_tracing_enabled_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "enabled = false").unwrap();

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert!(!settings.tracing.enabled);
    }

    #[test]
    fn parse_overrides_application_section_from_config_file() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[application]").unwrap();
        writeln!(file, "password = hunter2").unwrap();

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

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

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert_eq!(settings.web.port, 9200);
        assert_eq!(settings.tracing.level, lib_tracing::Levels::WARN);
    }

    #[test]
    fn parse_with_missing_config_file_fails() {
        let cli = crate::Cli {
            config: Some(std::path::PathBuf::from("/nonexistent/bitnode_console.conf")),
            ..Default::default()
        };
        let result = Settings::parse_with_cli(&cli);
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_invalid_port_value_fails() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[web]").unwrap();
        writeln!(file, "port = not_a_number").unwrap();

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let result = Settings::parse_with_cli(&cli);
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_invalid_tracing_level_fails() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = verbose").unwrap();

        let cli = crate::Cli { config: Some(file.path().to_path_buf()), ..Default::default() };
        let result = Settings::parse_with_cli(&cli);
        assert!(result.is_err());
    }

    #[test]
    fn cli_tracing_level_overrides_config_file_value() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[tracing]").unwrap();
        writeln!(file, "level = warn").unwrap();

        let cli = crate::Cli {
            config: Some(file.path().to_path_buf()),
            tracing_level: Some(lib_tracing::Levels::TRACE),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert_eq!(settings.tracing.level, lib_tracing::Levels::TRACE);
    }

    #[test]
    fn cli_tracing_enabled_overrides_default() {
        let cli = crate::Cli { tracing_enabled: Some(false), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert!(!settings.tracing.enabled);
    }

    #[test]
    fn cli_tracing_show_settings_startup_overrides_default() {
        let cli =
            crate::Cli { tracing_show_settings_startup: Some(true), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert!(settings.tracing.show_settings_startup);
    }

    #[test]
    fn cli_args_tracing_level_flag_flows_through_to_settings() {
        use clap::Parser as _;
        let cli = crate::Cli::try_parse_from(["bin", "--tracing-level", "trace"])
            .expect("CLI should parse --tracing-level trace");
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.tracing.level, lib_tracing::Levels::TRACE);
    }

    #[test]
    fn cli_args_config_flag_flows_through_to_settings() {
        use clap::Parser as _;
        use std::io::Write as _;
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[web]").unwrap();
        writeln!(file, "port = 9999").unwrap();
        writeln!(file, "host = 0.0.0.0").unwrap();

        let path = file.path().to_str().expect("path is valid UTF-8");
        let cli = crate::Cli::try_parse_from(["bin", "--config", path])
            .expect("CLI should parse --config");
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");

        assert_eq!(settings.web.port, 9999);
        assert_eq!(settings.web.host, "0.0.0.0");
    }

    #[test]
    fn cli_rpc_port_overrides_config_file_value() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[rpc]").unwrap();
        writeln!(file, "port = 50051").unwrap();
        writeln!(file, "host = 127.0.0.1").unwrap();
        writeln!(file, "password_hash = ").unwrap();
        writeln!(file, "token_secret = s").unwrap();

        let cli = crate::Cli {
            config: Some(file.path().to_path_buf()),
            rpc_port: Some(9090),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.rpc.port, 9090);
    }

    #[test]
    fn cli_rpc_host_overrides_config_file_value() {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        writeln!(file, "[rpc]").unwrap();
        writeln!(file, "port = 50051").unwrap();
        writeln!(file, "host = 127.0.0.1").unwrap();
        writeln!(file, "password_hash = ").unwrap();
        writeln!(file, "token_secret = s").unwrap();

        let cli = crate::Cli {
            config: Some(file.path().to_path_buf()),
            rpc_host: Some("0.0.0.0".to_string()),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.rpc.host, "0.0.0.0");
    }

    #[test]
    fn cli_rpc_allowed_ips_overrides_default() {
        let cli = crate::Cli {
            rpc_allowed_ips: Some(vec!["10.0.0.0/8".parse().unwrap()]),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        let expected: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
        assert_eq!(settings.rpc.allowed_ips(), &[expected]);
    }

    #[test]
    fn cli_rpc_flags_flow_through_from_parse_from() {
        use clap::Parser as _;
        let cli = crate::Cli::try_parse_from([
            "bin",
            "--rpc-port",
            "9090",
            "--rpc-host",
            "0.0.0.0",
        ])
        .expect("CLI should parse rpc flags");
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.rpc.port, 9090);
        assert_eq!(settings.rpc.host, "0.0.0.0");
    }

    #[test]
    fn cli_bitcoind_rpc_port_overrides_default() {
        let cli = crate::Cli { bitcoind_rpc_port: Some(18443), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.bitcoind.rpc_port(), 18443);
    }

    #[test]
    fn cli_bitcoind_rpc_host_overrides_default() {
        let cli = crate::Cli {
            bitcoind_rpc_host: Some("192.168.1.10".to_string()),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.bitcoind.rpc_host(), "192.168.1.10");
    }

    #[test]
    fn cli_bitcoind_unit_name_overrides_default() {
        let cli = crate::Cli {
            bitcoind_unit_name: Some("knots.service".to_string()),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.bitcoind.unit_name(), "knots.service");
    }

    #[test]
    fn cli_bitcoind_cookie_file_overrides_default() {
        let cli = crate::Cli {
            bitcoind_cookie_file: Some(std::path::PathBuf::from("/var/lib/bitcoind/.cookie")),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(
            settings.bitcoind.cookie_file(),
            Some(std::path::Path::new("/var/lib/bitcoind/.cookie"))
        );
    }

    #[test]
    fn cli_web_port_overrides_default() {
        let cli = crate::Cli { web_port: Some(3000), ..Default::default() };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.web.port, 3000);
    }

    #[test]
    fn cli_web_host_overrides_default() {
        let cli = crate::Cli {
            web_host: Some("0.0.0.0".to_string()),
            ..Default::default()
        };
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.web.host, "0.0.0.0");
    }

    #[test]
    fn cli_web_flags_flow_through_from_parse_from() {
        use clap::Parser as _;
        let cli =
            crate::Cli::try_parse_from(["bin", "--web-port", "3000", "--web-host", "0.0.0.0"])
                .expect("CLI should parse web flags");
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.web.port, 3000);
        assert_eq!(settings.web.host, "0.0.0.0");
    }

    #[test]
    fn cli_bitcoind_flags_flow_through_from_parse_from() {
        use clap::Parser as _;
        let cli = crate::Cli::try_parse_from([
            "bin",
            "--bitcoind-rpc-port",
            "18443",
            "--bitcoind-rpc-host",
            "192.168.1.10",
            "--bitcoind-rpc-user",
            "alice",
        ])
        .expect("CLI should parse bitcoind flags");
        let settings = Settings::parse_with_cli(&cli).expect("parse should succeed");
        assert_eq!(settings.bitcoind.rpc_port(), 18443);
        assert_eq!(settings.bitcoind.rpc_host(), "192.168.1.10");
        assert_eq!(settings.bitcoind.rpc_user(), "alice");
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
