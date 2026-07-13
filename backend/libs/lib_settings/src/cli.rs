//! Command-line argument definitions for the BitNode Console server binaries.
//!
//! [`Cli`] is parsed by [`Settings::parse`] and applied as the highest-priority
//! configuration source, overriding config files and environment variables.

use std::path::PathBuf;

/// Command-line arguments accepted by the BitNode Console server binaries.
///
/// These flags override all other configuration sources (config files,
/// environment variables, built-in defaults).
#[derive(Debug, Clone, PartialEq, Eq, clap::Parser)]
#[command(about)]
pub struct Cli {
    /// Path to a configuration file (overrides all config paths).
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the tracing enabled state.
    #[arg(long, value_name = "BOOL")]
    pub tracing_enabled: Option<bool>,

    /// Override the tracing level (off | error | warn | info | debug | trace).
    #[arg(short = 'l', long, value_name = "LEVEL")]
    pub tracing_level: Option<lib_tracing::Levels>,

    /// Override the tracing show-settings-on-startup state.
    #[arg(long, value_name = "BOOL")]
    pub tracing_show_settings_startup: Option<bool>,

    /// Override the RPC server port.
    #[arg(long, value_name = "PORT")]
    pub rpc_port: Option<u16>,

    /// Override the RPC server host address.
    #[arg(long, value_name = "HOST")]
    pub rpc_host: Option<String>,

    /// Override the RPC password hash (Argon2id PHC string).
    #[arg(long, value_name = "HASH")]
    pub rpc_password_hash: Option<String>,

    /// Override the RPC token signing secret.
    #[arg(long, value_name = "SECRET")]
    pub rpc_token_secret: Option<String>,

    /// Override the RPC allowed IP addresses and CIDR subnets.
    ///
    /// May be supplied multiple times or as a space-separated list.
    /// Replaces the entire allowed list when set.
    #[arg(long, value_name = "CIDR", num_args = 1..)]
    pub rpc_allowed_ips: Option<Vec<ipnet::IpNet>>,

    /// Override the Bitcoin daemon systemd unit name.
    #[arg(long, value_name = "UNIT")]
    pub bitcoind_unit_name: Option<String>,

    /// Override the Bitcoin daemon JSON-RPC host.
    #[arg(long, value_name = "HOST")]
    pub bitcoind_rpc_host: Option<String>,

    /// Override the Bitcoin daemon JSON-RPC port.
    #[arg(long, value_name = "PORT")]
    pub bitcoind_rpc_port: Option<u16>,

    /// Override the Bitcoin daemon JSON-RPC username.
    #[arg(long, value_name = "USER")]
    pub bitcoind_rpc_user: Option<String>,

    /// Override the Bitcoin daemon JSON-RPC password.
    #[arg(long, value_name = "PASSWORD")]
    pub bitcoind_rpc_password: Option<String>,

    /// Override the path to the Bitcoin daemon cookie file.
    #[arg(long, value_name = "FILE")]
    pub bitcoind_cookie_file: Option<PathBuf>,

    /// Override the web server port.
    #[arg(long, value_name = "PORT")]
    pub web_port: Option<u16>,

    /// Override the web server host address.
    #[arg(long, value_name = "HOST")]
    pub web_host: Option<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            config: None,
            tracing_enabled: None,
            tracing_level: None,
            tracing_show_settings_startup: None,
            rpc_port: None,
            rpc_host: None,
            rpc_password_hash: None,
            rpc_token_secret: None,
            rpc_allowed_ips: None,
            bitcoind_unit_name: None,
            bitcoind_rpc_host: None,
            bitcoind_rpc_port: None,
            bitcoind_rpc_user: None,
            bitcoind_rpc_password: None,
            bitcoind_cookie_file: None,
            web_port: None,
            web_host: None,
        }
    }
}

impl Cli {
    /// Creates a new [`Cli`] with default values (all flags unset).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the configured path to the configuration file, if set.
    #[must_use]
    pub fn config(&self) -> Option<&std::path::Path> {
        self.config.as_deref()
    }

    /// Returns the tracing enabled override, if set.
    #[must_use]
    pub fn tracing_enabled(&self) -> Option<bool> {
        self.tracing_enabled
    }

    /// Returns the tracing level override, if set.
    #[must_use]
    pub fn tracing_level(&self) -> Option<lib_tracing::Levels> {
        self.tracing_level
    }

    /// Returns the tracing show-settings-on-startup override, if set.
    #[must_use]
    pub fn tracing_show_settings_startup(&self) -> Option<bool> {
        self.tracing_show_settings_startup
    }

    /// Returns the RPC server port override, if set.
    #[must_use]
    pub fn rpc_port(&self) -> Option<u16> {
        self.rpc_port
    }

    /// Returns the RPC server host override, if set.
    #[must_use]
    pub fn rpc_host(&self) -> Option<&str> {
        self.rpc_host.as_deref()
    }

    /// Returns the RPC password hash override, if set.
    #[must_use]
    pub fn rpc_password_hash(&self) -> Option<&str> {
        self.rpc_password_hash.as_deref()
    }

    /// Returns the RPC token secret override, if set.
    #[must_use]
    pub fn rpc_token_secret(&self) -> Option<&str> {
        self.rpc_token_secret.as_deref()
    }

    /// Returns the RPC allowed IP networks override, if set.
    #[must_use]
    pub fn rpc_allowed_ips(&self) -> Option<&[ipnet::IpNet]> {
        self.rpc_allowed_ips.as_deref()
    }

    /// Returns the Bitcoin daemon systemd unit name override, if set.
    #[must_use]
    pub fn bitcoind_unit_name(&self) -> Option<&str> {
        self.bitcoind_unit_name.as_deref()
    }

    /// Returns the Bitcoin daemon JSON-RPC host override, if set.
    #[must_use]
    pub fn bitcoind_rpc_host(&self) -> Option<&str> {
        self.bitcoind_rpc_host.as_deref()
    }

    /// Returns the Bitcoin daemon JSON-RPC port override, if set.
    #[must_use]
    pub fn bitcoind_rpc_port(&self) -> Option<u16> {
        self.bitcoind_rpc_port
    }

    /// Returns the Bitcoin daemon JSON-RPC username override, if set.
    #[must_use]
    pub fn bitcoind_rpc_user(&self) -> Option<&str> {
        self.bitcoind_rpc_user.as_deref()
    }

    /// Returns the Bitcoin daemon JSON-RPC password override, if set.
    #[must_use]
    pub fn bitcoind_rpc_password(&self) -> Option<&str> {
        self.bitcoind_rpc_password.as_deref()
    }

    /// Returns the Bitcoin daemon cookie file path override, if set.
    #[must_use]
    pub fn bitcoind_cookie_file(&self) -> Option<&std::path::Path> {
        self.bitcoind_cookie_file.as_deref()
    }

    /// Returns the web server port override, if set.
    #[must_use]
    pub fn web_port(&self) -> Option<u16> {
        self.web_port
    }

    /// Returns the web server host override, if set.
    #[must_use]
    pub fn web_host(&self) -> Option<&str> {
        self.web_host.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    // --- default / new ---

    #[test]
    fn test_default() {
        let cli = Cli::default();
        assert!(cli.config.is_none());
        assert!(cli.tracing_enabled.is_none());
        assert!(cli.tracing_level.is_none());
        assert!(cli.tracing_show_settings_startup.is_none());
        assert!(cli.rpc_port.is_none());
        assert!(cli.rpc_host.is_none());
        assert!(cli.rpc_password_hash.is_none());
        assert!(cli.rpc_token_secret.is_none());
        assert!(cli.rpc_allowed_ips.is_none());
    }

    #[test]
    fn test_new() {
        let cli = Cli::new();
        assert!(cli.config.is_none());
        assert!(cli.tracing_enabled.is_none());
        assert!(cli.tracing_level.is_none());
        assert!(cli.tracing_show_settings_startup.is_none());
        assert!(cli.rpc_port.is_none());
        assert!(cli.rpc_host.is_none());
        assert!(cli.rpc_password_hash.is_none());
        assert!(cli.rpc_token_secret.is_none());
        assert!(cli.rpc_allowed_ips.is_none());
    }

    #[test]
    fn new_and_default_produce_equal_values() {
        assert_eq!(Cli::new(), Cli::default());
    }

    // --- clone / debug ---

    #[test]
    fn test_clone() {
        let cli = Cli {
            config: Some(PathBuf::from("/etc/bitnode.conf")),
            tracing_level: Some(lib_tracing::Levels::DEBUG),
            ..Cli::default()
        };
        assert_eq!(cli.clone(), cli);
    }

    #[test]
    fn test_debug_format() {
        let cli = Cli::new();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Cli"));
    }

    // --- config accessor ---

    #[test]
    fn default_config_is_none() {
        assert!(Cli::default().config().is_none());
    }

    #[test]
    fn config_accessor_returns_path() {
        let cli = Cli {
            config: Some(PathBuf::from("/etc/bitnode.conf")),
            ..Cli::default()
        };
        assert_eq!(cli.config(), Some(std::path::Path::new("/etc/bitnode.conf")));
    }

    // --- tracing_enabled accessor ---

    #[test]
    fn default_tracing_enabled_is_none() {
        assert!(Cli::default().tracing_enabled().is_none());
    }

    #[test]
    fn tracing_enabled_accessor_returns_value() {
        let cli = Cli { tracing_enabled: Some(false), ..Cli::default() };
        assert_eq!(cli.tracing_enabled(), Some(false));
    }

    // --- tracing_level accessor ---

    #[test]
    fn default_tracing_level_is_none() {
        assert!(Cli::default().tracing_level().is_none());
    }

    #[test]
    fn tracing_level_accessor_returns_level() {
        let cli = Cli {
            tracing_level: Some(lib_tracing::Levels::INFO),
            ..Cli::default()
        };
        assert_eq!(cli.tracing_level(), Some(lib_tracing::Levels::INFO));
    }

    // --- tracing_show_settings_startup accessor ---

    #[test]
    fn default_tracing_show_settings_startup_is_none() {
        assert!(Cli::default().tracing_show_settings_startup().is_none());
    }

    #[test]
    fn tracing_show_settings_startup_accessor_returns_value() {
        let cli = Cli { tracing_show_settings_startup: Some(true), ..Cli::default() };
        assert_eq!(cli.tracing_show_settings_startup(), Some(true));
    }

    // --- rpc_port accessor ---

    #[test]
    fn default_rpc_port_is_none() {
        assert!(Cli::default().rpc_port().is_none());
    }

    #[test]
    fn rpc_port_accessor_returns_value() {
        let cli = Cli { rpc_port: Some(9090), ..Cli::default() };
        assert_eq!(cli.rpc_port(), Some(9090));
    }

    // --- rpc_host accessor ---

    #[test]
    fn default_rpc_host_is_none() {
        assert!(Cli::default().rpc_host().is_none());
    }

    #[test]
    fn rpc_host_accessor_returns_value() {
        let cli = Cli { rpc_host: Some("0.0.0.0".to_string()), ..Cli::default() };
        assert_eq!(cli.rpc_host(), Some("0.0.0.0"));
    }

    // --- rpc_password_hash accessor ---

    #[test]
    fn default_rpc_password_hash_is_none() {
        assert!(Cli::default().rpc_password_hash().is_none());
    }

    #[test]
    fn rpc_password_hash_accessor_returns_value() {
        let cli = Cli {
            rpc_password_hash: Some("$argon2id$...".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.rpc_password_hash(), Some("$argon2id$..."));
    }

    // --- rpc_token_secret accessor ---

    #[test]
    fn default_rpc_token_secret_is_none() {
        assert!(Cli::default().rpc_token_secret().is_none());
    }

    #[test]
    fn rpc_token_secret_accessor_returns_value() {
        let cli = Cli {
            rpc_token_secret: Some("my-secret".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.rpc_token_secret(), Some("my-secret"));
    }

    // --- rpc_allowed_ips accessor ---

    #[test]
    fn default_rpc_allowed_ips_is_none() {
        assert!(Cli::default().rpc_allowed_ips().is_none());
    }

    #[test]
    fn rpc_allowed_ips_accessor_returns_slice() {
        let net: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
        let cli = Cli { rpc_allowed_ips: Some(vec![net]), ..Cli::default() };
        assert_eq!(cli.rpc_allowed_ips(), Some([net].as_slice()));
    }

    // --- bitcoind_unit_name accessor ---

    #[test]
    fn default_bitcoind_unit_name_is_none() {
        assert!(Cli::default().bitcoind_unit_name().is_none());
    }

    #[test]
    fn bitcoind_unit_name_accessor_returns_value() {
        let cli = Cli {
            bitcoind_unit_name: Some("knots.service".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.bitcoind_unit_name(), Some("knots.service"));
    }

    // --- bitcoind_rpc_host accessor ---

    #[test]
    fn default_bitcoind_rpc_host_is_none() {
        assert!(Cli::default().bitcoind_rpc_host().is_none());
    }

    #[test]
    fn bitcoind_rpc_host_accessor_returns_value() {
        let cli = Cli {
            bitcoind_rpc_host: Some("192.168.1.10".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.bitcoind_rpc_host(), Some("192.168.1.10"));
    }

    // --- bitcoind_rpc_port accessor ---

    #[test]
    fn default_bitcoind_rpc_port_is_none() {
        assert!(Cli::default().bitcoind_rpc_port().is_none());
    }

    #[test]
    fn bitcoind_rpc_port_accessor_returns_value() {
        let cli = Cli { bitcoind_rpc_port: Some(18443), ..Cli::default() };
        assert_eq!(cli.bitcoind_rpc_port(), Some(18443));
    }

    // --- bitcoind_rpc_user accessor ---

    #[test]
    fn default_bitcoind_rpc_user_is_none() {
        assert!(Cli::default().bitcoind_rpc_user().is_none());
    }

    #[test]
    fn bitcoind_rpc_user_accessor_returns_value() {
        let cli = Cli {
            bitcoind_rpc_user: Some("alice".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.bitcoind_rpc_user(), Some("alice"));
    }

    // --- bitcoind_rpc_password accessor ---

    #[test]
    fn default_bitcoind_rpc_password_is_none() {
        assert!(Cli::default().bitcoind_rpc_password().is_none());
    }

    #[test]
    fn bitcoind_rpc_password_accessor_returns_value() {
        let cli = Cli {
            bitcoind_rpc_password: Some("s3cr3t".to_string()),
            ..Cli::default()
        };
        assert_eq!(cli.bitcoind_rpc_password(), Some("s3cr3t"));
    }

    // --- bitcoind_cookie_file accessor ---

    #[test]
    fn default_bitcoind_cookie_file_is_none() {
        assert!(Cli::default().bitcoind_cookie_file().is_none());
    }

    #[test]
    fn bitcoind_cookie_file_accessor_returns_path() {
        let cli = Cli {
            bitcoind_cookie_file: Some(PathBuf::from("/var/lib/bitcoind/.cookie")),
            ..Cli::default()
        };
        assert_eq!(
            cli.bitcoind_cookie_file(),
            Some(std::path::Path::new("/var/lib/bitcoind/.cookie"))
        );
    }

    // --- web_port accessor ---

    #[test]
    fn default_web_port_is_none() {
        assert!(Cli::default().web_port().is_none());
    }

    #[test]
    fn web_port_accessor_returns_value() {
        let cli = Cli { web_port: Some(3000), ..Cli::default() };
        assert_eq!(cli.web_port(), Some(3000));
    }

    // --- web_host accessor ---

    #[test]
    fn default_web_host_is_none() {
        assert!(Cli::default().web_host().is_none());
    }

    #[test]
    fn web_host_accessor_returns_value() {
        let cli = Cli { web_host: Some("0.0.0.0".to_string()), ..Cli::default() };
        assert_eq!(cli.web_host(), Some("0.0.0.0"));
    }

    // --- parsing ---

    #[test]
    fn no_flags_leaves_all_fields_none() {
        let cli = Cli::try_parse_from(["bin"]).unwrap();
        assert!(cli.config.is_none());
        assert!(cli.tracing_enabled.is_none());
        assert!(cli.tracing_level.is_none());
        assert!(cli.tracing_show_settings_startup.is_none());
        assert!(cli.rpc_port.is_none());
        assert!(cli.rpc_host.is_none());
        assert!(cli.rpc_password_hash.is_none());
        assert!(cli.rpc_token_secret.is_none());
        assert!(cli.rpc_allowed_ips.is_none());
        assert!(cli.bitcoind_unit_name.is_none());
        assert!(cli.bitcoind_rpc_host.is_none());
        assert!(cli.bitcoind_rpc_port.is_none());
        assert!(cli.bitcoind_rpc_user.is_none());
        assert!(cli.bitcoind_rpc_password.is_none());
        assert!(cli.bitcoind_cookie_file.is_none());
        assert!(cli.web_port.is_none());
        assert!(cli.web_host.is_none());
    }

    #[test]
    fn long_config_flag_sets_path() {
        let cli = Cli::try_parse_from(["bin", "--config", "/etc/bitnode.conf"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/etc/bitnode.conf")));
    }

    #[test]
    fn short_config_flag_sets_path() {
        let cli = Cli::try_parse_from(["bin", "-c", "/etc/bitnode.conf"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/etc/bitnode.conf")));
    }

    #[test]
    fn tracing_enabled_flag_sets_true() {
        let cli = Cli::try_parse_from(["bin", "--tracing-enabled", "true"]).unwrap();
        assert_eq!(cli.tracing_enabled, Some(true));
    }

    #[test]
    fn tracing_enabled_flag_sets_false() {
        let cli = Cli::try_parse_from(["bin", "--tracing-enabled", "false"]).unwrap();
        assert_eq!(cli.tracing_enabled, Some(false));
    }

    #[test]
    fn long_tracing_level_flag_sets_level() {
        let cli = Cli::try_parse_from(["bin", "--tracing-level", "debug"]).unwrap();
        assert_eq!(cli.tracing_level, Some(lib_tracing::Levels::DEBUG));
    }

    #[test]
    fn short_tracing_level_flag_sets_level() {
        let cli = Cli::try_parse_from(["bin", "-l", "debug"]).unwrap();
        assert_eq!(cli.tracing_level, Some(lib_tracing::Levels::DEBUG));
    }

    #[test]
    fn all_tracing_level_values_parse() {
        let cases = [
            ("off", lib_tracing::Levels::OFF),
            ("error", lib_tracing::Levels::ERROR),
            ("warn", lib_tracing::Levels::WARN),
            ("info", lib_tracing::Levels::INFO),
            ("debug", lib_tracing::Levels::DEBUG),
            ("trace", lib_tracing::Levels::TRACE),
        ];
        for (input, expected) in cases {
            let cli = Cli::try_parse_from(["bin", "--tracing-level", input])
                .unwrap_or_else(|e| panic!("failed to parse level '{input}': {e}"));
            assert_eq!(cli.tracing_level, Some(expected));
        }
    }

    #[test]
    fn invalid_tracing_level_returns_error() {
        let result = Cli::try_parse_from(["bin", "--tracing-level", "verbose"]);
        assert!(result.is_err());
    }

    #[test]
    fn tracing_show_settings_startup_flag_sets_true() {
        let cli =
            Cli::try_parse_from(["bin", "--tracing-show-settings-startup", "true"]).unwrap();
        assert_eq!(cli.tracing_show_settings_startup, Some(true));
    }

    #[test]
    fn tracing_show_settings_startup_flag_sets_false() {
        let cli =
            Cli::try_parse_from(["bin", "--tracing-show-settings-startup", "false"]).unwrap();
        assert_eq!(cli.tracing_show_settings_startup, Some(false));
    }

    #[test]
    fn rpc_port_flag_sets_port() {
        let cli = Cli::try_parse_from(["bin", "--rpc-port", "9090"]).unwrap();
        assert_eq!(cli.rpc_port, Some(9090));
    }

    #[test]
    fn rpc_host_flag_sets_host() {
        let cli = Cli::try_parse_from(["bin", "--rpc-host", "0.0.0.0"]).unwrap();
        assert_eq!(cli.rpc_host, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn rpc_password_hash_flag_sets_hash() {
        let cli =
            Cli::try_parse_from(["bin", "--rpc-password-hash", "$argon2id$..."]).unwrap();
        assert_eq!(cli.rpc_password_hash, Some("$argon2id$...".to_string()));
    }

    #[test]
    fn rpc_token_secret_flag_sets_secret() {
        let cli =
            Cli::try_parse_from(["bin", "--rpc-token-secret", "my-secret"]).unwrap();
        assert_eq!(cli.rpc_token_secret, Some("my-secret".to_string()));
    }

    #[test]
    fn rpc_allowed_ips_flag_accepts_single_cidr() {
        let cli =
            Cli::try_parse_from(["bin", "--rpc-allowed-ips", "192.168.1.0/24"]).unwrap();
        let expected: ipnet::IpNet = "192.168.1.0/24".parse().unwrap();
        assert_eq!(cli.rpc_allowed_ips, Some(vec![expected]));
    }

    #[test]
    fn rpc_allowed_ips_flag_accepts_multiple_cidrs() {
        let cli = Cli::try_parse_from([
            "bin",
            "--rpc-allowed-ips",
            "192.168.1.0/24",
            "10.0.0.0/8",
        ])
        .unwrap();
        let expected: Vec<ipnet::IpNet> =
            ["192.168.1.0/24", "10.0.0.0/8"].iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(cli.rpc_allowed_ips, Some(expected));
    }

    #[test]
    fn invalid_rpc_port_returns_error() {
        let result = Cli::try_parse_from(["bin", "--rpc-port", "99999"]);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_rpc_allowed_ip_returns_error() {
        let result = Cli::try_parse_from(["bin", "--rpc-allowed-ips", "not-a-cidr"]);
        assert!(result.is_err());
    }

    #[test]
    fn config_and_tracing_level_can_be_combined() {
        let cli = Cli::try_parse_from([
            "bin",
            "--config",
            "/tmp/bitnode.conf",
            "--tracing-level",
            "warn",
        ])
        .unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/bitnode.conf")));
        assert_eq!(cli.tracing_level, Some(lib_tracing::Levels::WARN));
    }

    #[test]
    fn bitcoind_unit_name_flag_sets_value() {
        let cli =
            Cli::try_parse_from(["bin", "--bitcoind-unit-name", "knots.service"]).unwrap();
        assert_eq!(cli.bitcoind_unit_name, Some("knots.service".to_string()));
    }

    #[test]
    fn bitcoind_rpc_host_flag_sets_value() {
        let cli =
            Cli::try_parse_from(["bin", "--bitcoind-rpc-host", "192.168.1.10"]).unwrap();
        assert_eq!(cli.bitcoind_rpc_host, Some("192.168.1.10".to_string()));
    }

    #[test]
    fn bitcoind_rpc_port_flag_sets_value() {
        let cli = Cli::try_parse_from(["bin", "--bitcoind-rpc-port", "18443"]).unwrap();
        assert_eq!(cli.bitcoind_rpc_port, Some(18443));
    }

    #[test]
    fn bitcoind_rpc_user_flag_sets_value() {
        let cli = Cli::try_parse_from(["bin", "--bitcoind-rpc-user", "alice"]).unwrap();
        assert_eq!(cli.bitcoind_rpc_user, Some("alice".to_string()));
    }

    #[test]
    fn bitcoind_rpc_password_flag_sets_value() {
        let cli =
            Cli::try_parse_from(["bin", "--bitcoind-rpc-password", "s3cr3t"]).unwrap();
        assert_eq!(cli.bitcoind_rpc_password, Some("s3cr3t".to_string()));
    }

    #[test]
    fn bitcoind_cookie_file_flag_sets_path() {
        let cli = Cli::try_parse_from([
            "bin",
            "--bitcoind-cookie-file",
            "/var/lib/bitcoind/.cookie",
        ])
        .unwrap();
        assert_eq!(
            cli.bitcoind_cookie_file,
            Some(PathBuf::from("/var/lib/bitcoind/.cookie"))
        );
    }

    #[test]
    fn invalid_bitcoind_rpc_port_returns_error() {
        let result = Cli::try_parse_from(["bin", "--bitcoind-rpc-port", "99999"]);
        assert!(result.is_err());
    }

    #[test]
    fn web_port_flag_sets_value() {
        let cli = Cli::try_parse_from(["bin", "--web-port", "3000"]).unwrap();
        assert_eq!(cli.web_port, Some(3000));
    }

    #[test]
    fn web_host_flag_sets_value() {
        let cli = Cli::try_parse_from(["bin", "--web-host", "0.0.0.0"]).unwrap();
        assert_eq!(cli.web_host, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn invalid_web_port_returns_error() {
        let result = Cli::try_parse_from(["bin", "--web-port", "99999"]);
        assert!(result.is_err());
    }
}
