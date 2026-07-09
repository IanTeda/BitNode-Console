//-- ./backend/libs/lib_settings/src/bitcoind.rs

//! Bitcoin Daemon Settings
//!
//! This module contains configuration for the Bitcoin Knots/Core daemon,
//! covering both the systemd unit name used for log access and the JSON-RPC
//! credentials used for daemon communication.

use std::path::{Path, PathBuf};

use secrecy::{ExposeSecret, SecretString};

/// Default systemd unit name for the Bitcoin daemon.
const DEFAULT_UNIT_NAME: &str = "bitcoind.service";

/// Default RPC host.
const DEFAULT_RPC_HOST: &str = "127.0.0.1";

/// Default RPC port (Bitcoin mainnet).
const DEFAULT_RPC_PORT: u16 = 8332;

/// Default RPC username — empty; the server will reject RPC calls until configured.
const DEFAULT_RPC_USER: &str = "";

/// Default RPC password — empty; the server will reject RPC calls until configured.
const DEFAULT_RPC_PASSWORD: &str = "";

/// Serialises a [`SecretString`] as a plain string.
///
/// Required because [`SecretString`] intentionally omits a blanket [`serde::Serialize`]
/// impl to prevent accidental exposure. Used via `#[serde(serialize_with = …)]` so
/// that [`Settings`] can be serialised when seeding the [`config`] builder with
/// defaults.
fn serialize_secret_string<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

/// Bitcoin daemon configuration.
///
/// `Debug` output redacts `rpc_password` automatically (shows `[REDACTED]`).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BitcoinDaemonSettings {
    /// The systemd unit name of the Bitcoin daemon.
    ///
    /// Used to filter journal entries when serving logs over gRPC. Must match
    /// the `_SYSTEMD_UNIT` field written by the daemon's service unit, e.g.
    /// `"bitcoind.service"`. In development, when the daemon is launched via
    /// `systemd-cat -t bitcoind`, `lib_journals` also matches the
    /// `SYSLOG_IDENTIFIER` field derived by stripping the `.service` suffix.
    pub unit_name: String,

    /// The JSON-RPC host of the Bitcoin daemon.
    pub rpc_host: String,

    /// The JSON-RPC port of the Bitcoin daemon.
    ///
    /// Standard ports: mainnet `8332`, testnet `18332`, regtest `18443`.
    pub rpc_port: u16,

    /// The JSON-RPC username of the Bitcoin daemon.
    ///
    /// Must match the `rpcuser` value in `bitcoin.conf`. The default is empty,
    /// which causes all RPC calls to be rejected until a value is configured.
    pub rpc_user: String,

    /// The JSON-RPC password of the Bitcoin daemon.
    ///
    /// Must match the `rpcpassword` value in `bitcoin.conf`. Stored as a
    /// [`SecretString`] so the value is redacted in `Debug` output and zeroed
    /// in memory on drop.
    #[serde(serialize_with = "serialize_secret_string")]
    pub rpc_password: SecretString,

    /// Path to the Bitcoin daemon's `.cookie` file for cookie-based authentication.
    ///
    /// Bitcoin Core/Knots writes this file to `<datadir>/.cookie` on startup when
    /// `server=1` is set. When present, callers may use its contents as the RPC
    /// credential instead of `rpc_user`/`rpc_password`. Defaults to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie_file: Option<PathBuf>,
}

/// Manual [`PartialEq`] because [`SecretString`] intentionally omits it.
///
/// Compares the exposed password bytes directly; timing is not a concern here
/// since this is used only in tests and config comparisons, not for
/// authentication.
impl PartialEq for BitcoinDaemonSettings {
    fn eq(&self, other: &Self) -> bool {
        self.unit_name == other.unit_name
            && self.rpc_host == other.rpc_host
            && self.rpc_port == other.rpc_port
            && self.rpc_user == other.rpc_user
            && self.rpc_password.expose_secret() == other.rpc_password.expose_secret()
            && self.cookie_file == other.cookie_file
    }
}

impl Eq for BitcoinDaemonSettings {}

impl Default for BitcoinDaemonSettings {
    fn default() -> Self {
        Self {
            unit_name: DEFAULT_UNIT_NAME.to_string(),
            rpc_host: DEFAULT_RPC_HOST.to_string(),
            rpc_port: DEFAULT_RPC_PORT,
            rpc_user: DEFAULT_RPC_USER.to_string(),
            rpc_password: SecretString::from(DEFAULT_RPC_PASSWORD),
            cookie_file: None,
        }
    }
}

impl BitcoinDaemonSettings {
    /// Creates a new [`BitcoinDaemonSettings`] with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the configured systemd unit name.
    #[must_use]
    pub fn unit_name(&self) -> &str {
        &self.unit_name
    }

    /// Returns the RPC host.
    #[must_use]
    pub fn rpc_host(&self) -> &str {
        &self.rpc_host
    }

    /// Returns the RPC port.
    #[must_use]
    pub const fn rpc_port(&self) -> u16 {
        self.rpc_port
    }

    /// Returns the RPC username.
    #[must_use]
    pub fn rpc_user(&self) -> &str {
        &self.rpc_user
    }

    /// Returns the RPC password as a [`SecretString`].
    ///
    /// Call [`ExposeSecret::expose_secret`] on the returned value to access the
    /// raw string, e.g. `settings.rpc_password().expose_secret()`.
    #[must_use]
    pub const fn rpc_password(&self) -> &SecretString {
        &self.rpc_password
    }

    /// Returns the RPC address as a `"host:port"` string.
    #[must_use]
    pub fn rpc_address(&self) -> String {
        format!("{}:{}", self.rpc_host, self.rpc_port)
    }

    /// Returns the full HTTP URL for the JSON-RPC endpoint.
    #[must_use]
    pub fn rpc_url(&self) -> String {
        format!("http://{}:{}", self.rpc_host, self.rpc_port)
    }

    /// Returns the path to the Bitcoin daemon's `.cookie` file, if configured.
    #[must_use]
    pub fn cookie_file(&self) -> Option<&Path> {
        self.cookie_file.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> BitcoinDaemonSettings {
        BitcoinDaemonSettings {
            unit_name: "bitcoind.service".to_string(),
            rpc_host: "192.168.1.10".to_string(),
            rpc_port: 18443,
            rpc_user: "alice".to_string(),
            rpc_password: SecretString::from("s3cr3t"),
            cookie_file: None,
        }
    }

    // --- default / new ---

    #[test]
    fn default_unit_name_is_bitcoind_service() {
        assert_eq!(
            BitcoinDaemonSettings::default().unit_name(),
            DEFAULT_UNIT_NAME
        );
    }

    #[test]
    fn default_rpc_host_is_localhost() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_host(),
            DEFAULT_RPC_HOST
        );
    }

    #[test]
    fn default_rpc_port_is_mainnet_port() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_port(),
            DEFAULT_RPC_PORT
        );
    }

    #[test]
    fn default_rpc_user_is_empty() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_user(),
            DEFAULT_RPC_USER
        );
    }

    #[test]
    fn default_rpc_password_is_empty() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_password().expose_secret(),
            DEFAULT_RPC_PASSWORD
        );
    }

    #[test]
    fn new_returns_same_values_as_default() {
        assert_eq!(
            BitcoinDaemonSettings::new(),
            BitcoinDaemonSettings::default()
        );
    }

    // --- accessors ---

    #[test]
    fn unit_name_accessor_returns_field_value() {
        let s = BitcoinDaemonSettings {
            unit_name: "knots.service".to_string(),
            ..BitcoinDaemonSettings::default()
        };
        assert_eq!(s.unit_name(), "knots.service");
    }

    #[test]
    fn rpc_host_accessor_returns_field_value() {
        assert_eq!(settings().rpc_host(), "192.168.1.10");
    }

    #[test]
    fn rpc_port_accessor_returns_field_value() {
        assert_eq!(settings().rpc_port(), 18443);
    }

    #[test]
    fn rpc_user_accessor_returns_field_value() {
        assert_eq!(settings().rpc_user(), "alice");
    }

    #[test]
    fn rpc_password_accessor_returns_secret_string() {
        assert_eq!(settings().rpc_password().expose_secret(), "s3cr3t");
    }

    #[test]
    fn rpc_password_debug_is_redacted() {
        let s = settings();
        let debug = format!("{s:?}");
        assert!(
            !debug.contains("s3cr3t"),
            "password leaked in Debug: {debug}"
        );
        assert!(
            debug.contains("[REDACTED]"),
            "expected [REDACTED] in: {debug}"
        );
    }

    // --- rpc_address ---

    #[test]
    fn rpc_address_formats_host_and_port() {
        assert_eq!(settings().rpc_address(), "192.168.1.10:18443");
    }

    #[test]
    fn rpc_address_default_is_localhost_mainnet() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_address(),
            format!("{DEFAULT_RPC_HOST}:{DEFAULT_RPC_PORT}")
        );
    }

    #[test]
    fn rpc_address_with_port_zero() {
        let s = BitcoinDaemonSettings {
            rpc_port: 0,
            ..BitcoinDaemonSettings::default()
        };
        assert_eq!(s.rpc_address(), "127.0.0.1:0");
    }

    #[test]
    fn rpc_address_with_max_port() {
        let s = BitcoinDaemonSettings {
            rpc_port: u16::MAX,
            ..BitcoinDaemonSettings::default()
        };
        assert_eq!(s.rpc_address(), format!("127.0.0.1:{}", u16::MAX));
    }

    // --- rpc_url ---

    #[test]
    fn rpc_url_includes_http_scheme() {
        assert!(settings().rpc_url().starts_with("http://"));
    }

    #[test]
    fn rpc_url_formats_correctly() {
        assert_eq!(settings().rpc_url(), "http://192.168.1.10:18443");
    }

    #[test]
    fn rpc_url_default_is_localhost_mainnet() {
        assert_eq!(
            BitcoinDaemonSettings::default().rpc_url(),
            format!("http://{DEFAULT_RPC_HOST}:{DEFAULT_RPC_PORT}")
        );
    }

    #[test]
    fn rpc_url_with_ipv6_host() {
        let s = BitcoinDaemonSettings {
            rpc_host: "::1".to_string(),
            rpc_port: 8332,
            ..BitcoinDaemonSettings::default()
        };
        assert_eq!(s.rpc_url(), "http://::1:8332");
    }

    // --- clone / debug / eq ---

    #[test]
    fn clone_produces_equal_value() {
        let s = settings();
        assert_eq!(s.clone(), s);
    }

    #[test]
    fn debug_format_includes_struct_name() {
        let debug = format!("{:?}", BitcoinDaemonSettings::default());
        assert!(debug.contains("BitcoinDaemonSettings"));
    }

    #[test]
    fn debug_format_includes_unit_name() {
        let debug = format!("{:?}", BitcoinDaemonSettings::default());
        assert!(debug.contains(DEFAULT_UNIT_NAME));
    }

    #[test]
    fn debug_format_does_not_expose_default_password() {
        let debug = format!("{:?}", BitcoinDaemonSettings::default());
        // Even the empty default should not appear as a bare string value.
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn equal_settings_compare_equal() {
        assert_eq!(
            BitcoinDaemonSettings::default(),
            BitcoinDaemonSettings::default()
        );
    }

    #[test]
    fn different_unit_names_compare_unequal() {
        let a = BitcoinDaemonSettings {
            unit_name: "bitcoind.service".to_string(),
            ..BitcoinDaemonSettings::default()
        };
        let b = BitcoinDaemonSettings {
            unit_name: "knots.service".to_string(),
            ..BitcoinDaemonSettings::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn different_rpc_ports_compare_unequal() {
        let a = BitcoinDaemonSettings {
            rpc_port: 8332,
            ..BitcoinDaemonSettings::default()
        };
        let b = BitcoinDaemonSettings {
            rpc_port: 18443,
            ..BitcoinDaemonSettings::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn different_passwords_compare_unequal() {
        let a = BitcoinDaemonSettings {
            rpc_password: SecretString::from("alpha"),
            ..BitcoinDaemonSettings::default()
        };
        let b = BitcoinDaemonSettings {
            rpc_password: SecretString::from("bravo"),
            ..BitcoinDaemonSettings::default()
        };
        assert_ne!(a, b);
    }

    // --- serialisation ---

    #[test]
    fn serialize_deserialize_roundtrip() {
        let s = settings();
        let json = serde_json::to_string(&s).expect("serialize BitcoinDaemonSettings");
        let deserialized: BitcoinDaemonSettings =
            serde_json::from_str(&json).expect("deserialize BitcoinDaemonSettings");
        assert_eq!(s, deserialized);
    }

    #[test]
    fn serialize_produces_expected_json_field_names() {
        let json = serde_json::to_string(&settings()).expect("serialize BitcoinDaemonSettings");
        assert!(
            json.contains("\"unit_name\""),
            "missing 'unit_name': {json}"
        );
        assert!(json.contains("\"rpc_host\""), "missing 'rpc_host': {json}");
        assert!(json.contains("\"rpc_port\""), "missing 'rpc_port': {json}");
        assert!(json.contains("\"rpc_user\""), "missing 'rpc_user': {json}");
        assert!(
            json.contains("\"rpc_password\""),
            "missing 'rpc_password': {json}"
        );
    }

    #[test]
    fn serialize_exposes_password_in_json() {
        // Serialisation must write the real value so the config round-trip works.
        let json = serde_json::to_string(&settings()).expect("serialize");
        assert!(
            json.contains("s3cr3t"),
            "password missing from JSON: {json}"
        );
    }

    #[test]
    fn deserialize_missing_all_fields_fails() {
        assert!(serde_json::from_str::<BitcoinDaemonSettings>("{}").is_err());
    }

    #[test]
    fn deserialize_missing_unit_name_fails() {
        let json = r#"{"rpc_host":"127.0.0.1","rpc_port":8332,"rpc_user":"u","rpc_password":"p"}"#;
        assert!(serde_json::from_str::<BitcoinDaemonSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_rpc_host_fails() {
        let json =
            r#"{"unit_name":"bitcoind.service","rpc_port":8332,"rpc_user":"u","rpc_password":"p"}"#;
        assert!(serde_json::from_str::<BitcoinDaemonSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_rpc_port_fails() {
        let json = r#"{"unit_name":"bitcoind.service","rpc_host":"127.0.0.1","rpc_user":"u","rpc_password":"p"}"#;
        assert!(serde_json::from_str::<BitcoinDaemonSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_rpc_user_fails() {
        let json = r#"{"unit_name":"bitcoind.service","rpc_host":"127.0.0.1","rpc_port":8332,"rpc_password":"p"}"#;
        assert!(serde_json::from_str::<BitcoinDaemonSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_rpc_password_fails() {
        let json = r#"{"unit_name":"bitcoind.service","rpc_host":"127.0.0.1","rpc_port":8332,"rpc_user":"u"}"#;
        assert!(serde_json::from_str::<BitcoinDaemonSettings>(json).is_err());
    }

    #[test]
    fn deserialize_full_json_object_succeeds() {
        let json = r#"{
            "unit_name": "bitcoind.service",
            "rpc_host": "127.0.0.1",
            "rpc_port": 8332,
            "rpc_user": "bitcoinrpc",
            "rpc_password": "devpassword"
        }"#;
        let s: BitcoinDaemonSettings = serde_json::from_str(json).expect("deserialize full object");
        assert_eq!(s.unit_name(), "bitcoind.service");
        assert_eq!(s.rpc_host(), "127.0.0.1");
        assert_eq!(s.rpc_port(), 8332);
        assert_eq!(s.rpc_user(), "bitcoinrpc");
        assert_eq!(s.rpc_password().expose_secret(), "devpassword");
    }

    #[test]
    fn deserialize_password_is_wrapped_as_secret() {
        let json = r#"{"unit_name":"u","rpc_host":"h","rpc_port":1,"rpc_user":"u","rpc_password":"hunter2"}"#;
        let s: BitcoinDaemonSettings = serde_json::from_str(json).expect("deserialize");
        // Value is accessible via expose_secret, not a plain String.
        assert_eq!(s.rpc_password().expose_secret(), "hunter2");
        assert!(!format!("{s:?}").contains("hunter2"));
    }

    #[test]
    fn serialize_preserves_regtest_port() {
        let s = BitcoinDaemonSettings {
            rpc_port: 18443,
            ..BitcoinDaemonSettings::default()
        };
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(json.contains("18443"), "regtest port missing: {json}");
    }

    // --- cookie_file ---

    #[test]
    fn default_cookie_file_is_none() {
        assert!(BitcoinDaemonSettings::default().cookie_file().is_none());
    }

    #[test]
    fn cookie_file_accessor_returns_configured_path() {
        let s = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/var/lib/bitcoind/.cookie")),
            ..BitcoinDaemonSettings::default()
        };
        assert_eq!(
            s.cookie_file(),
            Some(Path::new("/var/lib/bitcoind/.cookie"))
        );
    }

    #[test]
    fn cookie_file_none_compares_equal() {
        let a = BitcoinDaemonSettings::default();
        let b = BitcoinDaemonSettings::default();
        assert_eq!(a, b);
    }

    #[test]
    fn different_cookie_files_compare_unequal() {
        let a = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/a/.cookie")),
            ..BitcoinDaemonSettings::default()
        };
        let b = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/b/.cookie")),
            ..BitcoinDaemonSettings::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn some_and_none_cookie_files_compare_unequal() {
        let a = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/var/lib/bitcoind/.cookie")),
            ..BitcoinDaemonSettings::default()
        };
        let b = BitcoinDaemonSettings::default();
        assert_ne!(a, b);
    }

    #[test]
    fn serialize_omits_cookie_file_when_none() {
        let json = serde_json::to_string(&BitcoinDaemonSettings::default()).expect("serialize");
        assert!(
            !json.contains("cookie_file"),
            "cookie_file should be omitted when None: {json}"
        );
    }

    #[test]
    fn serialize_includes_cookie_file_when_some() {
        let s = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/var/lib/bitcoind/.cookie")),
            ..BitcoinDaemonSettings::default()
        };
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(
            json.contains("cookie_file"),
            "cookie_file missing from JSON: {json}"
        );
        assert!(
            json.contains("/var/lib/bitcoind/.cookie"),
            "cookie path missing from JSON: {json}"
        );
    }

    #[test]
    fn deserialize_without_cookie_file_defaults_to_none() {
        let json = r#"{
            "unit_name": "bitcoind.service",
            "rpc_host": "127.0.0.1",
            "rpc_port": 8332,
            "rpc_user": "u",
            "rpc_password": "p"
        }"#;
        let s: BitcoinDaemonSettings = serde_json::from_str(json).expect("deserialize");
        assert!(s.cookie_file().is_none());
    }

    #[test]
    fn deserialize_with_cookie_file_succeeds() {
        let json = r#"{
            "unit_name": "bitcoind.service",
            "rpc_host": "127.0.0.1",
            "rpc_port": 8332,
            "rpc_user": "u",
            "rpc_password": "p",
            "cookie_file": "/var/lib/bitcoind/.cookie"
        }"#;
        let s: BitcoinDaemonSettings = serde_json::from_str(json).expect("deserialize");
        assert_eq!(
            s.cookie_file(),
            Some(Path::new("/var/lib/bitcoind/.cookie"))
        );
    }

    #[test]
    fn serialize_deserialize_roundtrip_with_cookie_file() {
        let s = BitcoinDaemonSettings {
            cookie_file: Some(PathBuf::from("/tmp/.cookie")),
            ..settings()
        };
        let json = serde_json::to_string(&s).expect("serialize");
        let deserialized: BitcoinDaemonSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s, deserialized);
    }
}
