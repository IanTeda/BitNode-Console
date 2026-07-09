//-- ./backend/libs/lib_settings/src/application.rs

//! Application Settings
//!
//! This module contains the application settings struct and related functions.

/// Default value for the `password` field.
const DEFAULT_PASSWORD: &str = "";

/// Application-level settings.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ApplicationSettings {
    /// The password used for application authentication.
    pub password: String,
}

// Default implementation for ApplicationSettings.
impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            password: DEFAULT_PASSWORD.to_string(),
        }
    }
}

impl ApplicationSettings {
    /// Creates a new [`ApplicationSettings`] with the default password.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the configured password.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_empty_password() {
        let settings = ApplicationSettings::default();
        assert_eq!(settings.password(), DEFAULT_PASSWORD);
    }

    #[test]
    fn password_accessor_returns_field_value() {
        let settings = ApplicationSettings {
            password: "hunter2".to_string(),
        };
        assert_eq!(settings.password(), "hunter2");
    }

    #[test]
    fn clone_produces_equal_value() {
        let settings = ApplicationSettings {
            password: "secret".to_string(),
        };
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn debug_format_includes_struct_name() {
        let settings = ApplicationSettings::default();
        let debug_str = format!("{settings:?}");
        assert!(debug_str.contains("ApplicationSettings"));
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let settings = ApplicationSettings {
            password: "my_password".to_string(),
        };
        let json = serde_json::to_string(&settings).expect("serialize ApplicationSettings");
        let deserialized: ApplicationSettings =
            serde_json::from_str(&json).expect("deserialize ApplicationSettings");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn serialize_produces_expected_json_field_names() {
        let settings = ApplicationSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize ApplicationSettings");
        assert!(
            json.contains("\"password\""),
            "missing 'password' field: {json}"
        );
    }

    #[test]
    fn deserialize_missing_field_fails() {
        let result: Result<ApplicationSettings, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn different_passwords_compare_unequal() {
        let a = ApplicationSettings {
            password: "alpha".to_string(),
        };
        let b = ApplicationSettings {
            password: "bravo".to_string(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn equal_settings_compare_equal() {
        let a = ApplicationSettings::default();
        let b = ApplicationSettings::default();
        assert_eq!(a, b);
    }
}
