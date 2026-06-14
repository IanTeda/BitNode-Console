//! Server Configuration
//! This module contains the server configuration struct and related functions.

/// Application settings for the server binary, with [`ServerSettings`] as
/// the server-specific configuration section.
pub type Settings = lib_settings::Settings<ServerSettings>;

/// Default port for the server
const DEFAULT_PORT: u16 = 8080;

/// Default host for the server
const DEFAULT_HOST: &str = "127.0.0.1";

/// Server configuration struct
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerSettings {
    /// Server port
    pub port: u16,

    /// Server host
    pub host: String,
}

/// Default implementation for ServerSettings
impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: DEFAULT_HOST.to_string(),
        }
    }
}

impl ServerSettings {
    /// Creates a new ServerSettings with the default port and host
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the server address as a string
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        let config = ServerSettings {
            port: 8080,
            host: "localhost".to_string(),
        };
        assert_eq!(config.address(), "localhost:8080");
    }

    #[test]
    fn test_default() {
        let config = ServerSettings::default();
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.host, DEFAULT_HOST);
    }

    #[test]
    fn test_new() {
        let config = ServerSettings::new();
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.host, DEFAULT_HOST);
    }

    #[test]
    fn test_address_with_default() {
        let config = ServerSettings::default();
        assert_eq!(config.address(), format!("{DEFAULT_HOST}:{DEFAULT_PORT}"));
    }

    #[test]
    fn test_clone() {
        let config = ServerSettings::new();
        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.port, cloned.port);
    }

    #[test]
    fn test_debug_format() {
        let config = ServerSettings::new();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("ServerSettings"));
        assert!(debug_str.contains(DEFAULT_HOST));
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = ServerSettings {
            port: 9000,
            host: "0.0.0.0".to_string(),
        };

        let json = serde_json::to_string(&config).expect("serialize ServerSettings");
        let deserialized: ServerSettings =
            serde_json::from_str(&json).expect("deserialize ServerSettings");

        assert_eq!(deserialized.port, config.port);
        assert_eq!(deserialized.host, config.host);
    }

    #[test]
    fn test_deserialize_missing_fields_fails() {
        let json = "{}";
        let result: Result<ServerSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
