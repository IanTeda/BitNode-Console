//-- ./backend/libs/lib_settings/src/rpc.rs

//! RPC Settings
//! This module contains the RPC settings struct and related functions.

/// Default port for the server.
const DEFAULT_PORT: u16 = 50051;

/// Default host for the server.
const DEFAULT_HOST: &str = "127.0.0.1";

/// Default password hash — empty; server will reject all logins until configured.
const DEFAULT_PASSWORD_HASH: &str = "";

/// Default token secret — empty; server will fail to sign tokens until configured.
const DEFAULT_TOKEN_SECRET: &str = "";

/// RPC server configuration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RpcSettings {
    /// Server port.
    pub port: u16,

    /// Server host.
    pub host: String,

    /// Argon2id PHC hash string of the application password.
    ///
    /// The default is empty, which causes all login attempts to fail.
    /// Generate a value at the command line and paste the output here:
    ///
    /// ```text
    /// $ sudo apt install libargon2-0 -y && \
    ///   echo -n "yourpassword" | argon2 "$(openssl rand -hex 16)" -id -e
    /// ```
    ///
    /// The output is a `$argon2id$…` PHC string that the server parses on
    /// startup and uses to verify every login request.
    pub password_hash: String,

    /// Secret key used to sign and verify JWT access and refresh tokens.
    ///
    /// Must be a strong random value in production. The default is empty and
    /// will cause token signing to fail until a real value is configured.
    pub token_secret: String,
}

impl Default for RpcSettings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: DEFAULT_HOST.to_string(),
            password_hash: DEFAULT_PASSWORD_HASH.to_string(),
            token_secret: DEFAULT_TOKEN_SECRET.to_string(),
        }
    }
}

impl RpcSettings {
    /// Creates a new [`RpcSettings`] with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the server address as a `"host:port"` string.
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the stored Argon2id PHC hash string.
    #[must_use]
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    /// Returns the token signing secret.
    #[must_use]
    pub fn token_secret(&self) -> &str {
        &self.token_secret
    }

    /// Returns the server socket address parsed from the host and port.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error::Parsing`] if the host string is not a valid IP address.
    pub fn socket_address(&self) -> crate::Result<std::net::SocketAddr> {
        let ip = self
            .host
            .parse()
            .map_err(|e: std::net::AddrParseError| crate::Error::Parsing(e.to_string()))?;
        Ok(std::net::SocketAddr::new(ip, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A well-formed Argon2id PHC string used in tests that need a valid hash value.
    const SAMPLE_HASH: &str = "$argon2id$v=19$m=15000,t=2,p=1$c29tZXNhbHQ$bWh0ZSB0ZXN0IGhhc2g";

    fn settings() -> RpcSettings {
        RpcSettings {
            port: 8080,
            host: "127.0.0.1".to_string(),
            password_hash: SAMPLE_HASH.to_string(),
            token_secret: "supersecret".to_string(),
        }
    }

    // --- address ---

    #[test]
    fn test_address() {
        let settings = RpcSettings {
            port: 8080,
            host: "localhost".to_string(),
            password_hash: String::new(),
            token_secret: String::new(),
        };
        assert_eq!(settings.address(), "localhost:8080");
    }

    #[test]
    fn test_address_with_default() {
        let settings = RpcSettings::default();
        assert_eq!(settings.address(), format!("{DEFAULT_HOST}:{DEFAULT_PORT}"));
    }

    #[test]
    fn address_with_port_zero() {
        let settings = RpcSettings {
            port: 0,
            host: "127.0.0.1".to_string(),
            password_hash: String::new(),
            token_secret: String::new(),
        };
        assert_eq!(settings.address(), "127.0.0.1:0");
    }

    #[test]
    fn address_with_max_port() {
        let settings = RpcSettings {
            port: u16::MAX,
            host: "127.0.0.1".to_string(),
            password_hash: String::new(),
            token_secret: String::new(),
        };
        assert_eq!(settings.address(), format!("127.0.0.1:{}", u16::MAX));
    }

    #[test]
    fn address_with_ipv6_host() {
        let settings = RpcSettings {
            port: 8080,
            host: "::1".to_string(),
            password_hash: String::new(),
            token_secret: String::new(),
        };
        assert_eq!(settings.address(), "::1:8080");
    }

    // --- default / new ---

    #[test]
    fn test_default() {
        let s = RpcSettings::default();
        assert_eq!(s.port, DEFAULT_PORT);
        assert_eq!(s.host, DEFAULT_HOST);
        assert_eq!(s.password_hash, DEFAULT_PASSWORD_HASH);
        assert_eq!(s.token_secret, DEFAULT_TOKEN_SECRET);
    }

    #[test]
    fn test_new() {
        let s = RpcSettings::new();
        assert_eq!(s.port, DEFAULT_PORT);
        assert_eq!(s.host, DEFAULT_HOST);
        assert_eq!(s.password_hash, DEFAULT_PASSWORD_HASH);
        assert_eq!(s.token_secret, DEFAULT_TOKEN_SECRET);
    }

    #[test]
    fn new_and_default_produce_equal_values() {
        assert_eq!(RpcSettings::new(), RpcSettings::default());
    }

    // --- clone / debug ---

    #[test]
    fn test_clone() {
        let s = settings();
        assert_eq!(s.clone(), s);
    }

    #[test]
    fn test_debug_format() {
        let s = RpcSettings::new();
        let debug = format!("{s:?}");
        assert!(debug.contains("RpcSettings"));
        assert!(debug.contains(DEFAULT_HOST));
    }

    // --- password_hash ---

    #[test]
    fn default_password_hash_is_empty() {
        assert_eq!(
            RpcSettings::default().password_hash(),
            DEFAULT_PASSWORD_HASH
        );
    }

    #[test]
    fn password_hash_accessor_returns_field_value() {
        assert_eq!(settings().password_hash(), SAMPLE_HASH);
    }

    #[test]
    fn password_hash_stores_arbitrary_string() {
        let s = RpcSettings {
            password_hash: "some-phc-string".to_string(),
            ..RpcSettings::default()
        };
        assert_eq!(s.password_hash(), "some-phc-string");
    }

    // --- token_secret ---

    #[test]
    fn default_token_secret_is_empty() {
        assert_eq!(RpcSettings::default().token_secret(), DEFAULT_TOKEN_SECRET);
    }

    #[test]
    fn token_secret_accessor_returns_field_value() {
        assert_eq!(settings().token_secret(), "supersecret");
    }

    #[test]
    fn token_secret_stores_arbitrary_value() {
        let s = RpcSettings {
            token_secret: "my-jwt-signing-key".to_string(),
            ..RpcSettings::default()
        };
        assert_eq!(s.token_secret(), "my-jwt-signing-key");
    }

    // --- serialisation ---

    #[test]
    fn test_serialize_deserialize() {
        let s = settings();
        let json = serde_json::to_string(&s).expect("serialize RpcSettings");
        let deserialized: RpcSettings =
            serde_json::from_str(&json).expect("deserialize RpcSettings");
        assert_eq!(s, deserialized);
    }

    #[test]
    fn serialize_produces_expected_json_field_names() {
        let json = serde_json::to_string(&settings()).expect("serialize RpcSettings");
        assert!(json.contains("\"port\""), "missing 'port': {json}");
        assert!(json.contains("\"host\""), "missing 'host': {json}");
        assert!(
            json.contains("\"password_hash\""),
            "missing 'password_hash': {json}"
        );
        assert!(
            json.contains("\"token_secret\""),
            "missing 'token_secret': {json}"
        );
    }

    #[test]
    fn test_deserialize_missing_fields_fails() {
        assert!(serde_json::from_str::<RpcSettings>("{}").is_err());
    }

    #[test]
    fn deserialize_missing_host_fails() {
        let json = r#"{"port": 8080, "password_hash": "", "token_secret": "s"}"#;
        assert!(serde_json::from_str::<RpcSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_port_fails() {
        let json = r#"{"host": "127.0.0.1", "password_hash": "", "token_secret": "s"}"#;
        assert!(serde_json::from_str::<RpcSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_password_hash_fails() {
        let json = r#"{"host": "127.0.0.1", "port": 50051, "token_secret": "s"}"#;
        assert!(serde_json::from_str::<RpcSettings>(json).is_err());
    }

    #[test]
    fn deserialize_missing_token_secret_fails() {
        let json = r#"{"host": "127.0.0.1", "port": 50051, "password_hash": ""}"#;
        assert!(serde_json::from_str::<RpcSettings>(json).is_err());
    }
}
