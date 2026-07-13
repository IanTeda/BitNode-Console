//-- ./backend/libs/lib_settings/src/frontend.rs

//! Frontend Settings
//! This module contains the frontend settings struct and related functions.

/// Default port for the server
const DEFAULT_PORT: u16 = 8090;

/// Default host for the server
const DEFAULT_HOST: &str = "127.0.0.1";

/// Frontend settings struct
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FrontendSettings {
    /// Server port
    pub port: u16,

    /// Server host
    pub host: String,
}

impl Default for FrontendSettings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: DEFAULT_HOST.to_string(),
        }
    }
}

impl FrontendSettings {
    /// Creates a new [`FrontendSettings`] with the default port and host.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the server address as a `"host:port"` string.
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        let settings = FrontendSettings {
            port: 8080,
            host: "localhost".to_string(),
        };
        assert_eq!(settings.address(), "localhost:8080");
    }

    #[test]
    fn test_default() {
        let settings = FrontendSettings::default();
        assert_eq!(settings.port, DEFAULT_PORT);
        assert_eq!(settings.host, DEFAULT_HOST);
    }

    #[test]
    fn test_new() {
        let settings = FrontendSettings::new();
        assert_eq!(settings.port, DEFAULT_PORT);
        assert_eq!(settings.host, DEFAULT_HOST);
    }

    #[test]
    fn test_address_with_default() {
        let settings = FrontendSettings::default();
        assert_eq!(settings.address(), format!("{DEFAULT_HOST}:{DEFAULT_PORT}"));
    }

    #[test]
    fn test_clone() {
        let settings = FrontendSettings::new();
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn new_and_default_produce_equal_values() {
        assert_eq!(FrontendSettings::new(), FrontendSettings::default());
    }

    #[test]
    fn test_debug_format() {
        let settings = FrontendSettings::new();
        let debug_str = format!("{settings:?}");
        assert!(debug_str.contains("FrontendSettings"));
        assert!(debug_str.contains(DEFAULT_HOST));
    }

    #[test]
    fn address_with_port_zero() {
        let settings = FrontendSettings {
            port: 0,
            host: "127.0.0.1".to_string(),
        };
        assert_eq!(settings.address(), "127.0.0.1:0");
    }

    #[test]
    fn address_with_max_port() {
        let settings = FrontendSettings {
            port: u16::MAX,
            host: "127.0.0.1".to_string(),
        };
        assert_eq!(settings.address(), format!("127.0.0.1:{}", u16::MAX));
    }

    #[test]
    fn address_with_ipv6_host() {
        let settings = FrontendSettings {
            port: 8080,
            host: "::1".to_string(),
        };
        assert_eq!(settings.address(), "::1:8080");
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = FrontendSettings {
            port: 9000,
            host: "0.0.0.0".to_string(),
        };
        let json = serde_json::to_string(&settings).expect("serialize FrontendSettings");
        let deserialized: FrontendSettings =
            serde_json::from_str(&json).expect("deserialize FrontendSettings");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn serialize_produces_expected_json_field_names() {
        let settings = FrontendSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize FrontendSettings");
        assert!(json.contains("\"port\""), "missing 'port' field: {json}");
        assert!(json.contains("\"host\""), "missing 'host' field: {json}");
    }

    #[test]
    fn test_deserialize_missing_fields_fails() {
        let result: Result<FrontendSettings, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_missing_host_fails() {
        let result: Result<FrontendSettings, _> = serde_json::from_str(r#"{"port": 8080}"#);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_missing_port_fails() {
        let result: Result<FrontendSettings, _> =
            serde_json::from_str(r#"{"host": "127.0.0.1"}"#);
        assert!(result.is_err());
    }
}
