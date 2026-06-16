//-- ./backend/libs/lib_settings/src/application.rs

//! Application Settings
//!
//! This module contains the application settings struct and related functions.

/// Default value for the `log_settings` flag.
const DEFAULT_SETTINGS: bool = false;

/// Application-level settings.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ApplicationSettings {
    /// Log the active settings to the tracing output at startup.
    pub setting: bool,
}

// Default implementation for ApplicationSettings.
impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            setting: DEFAULT_SETTINGS,
        }
    }
}

impl ApplicationSettings {
    /// Returns `true` if the active settings should be logged at startup.
    #[must_use]
    pub const fn setting(&self) -> bool {
        self.setting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disables_log_settings() {
        let settings = ApplicationSettings::default();
        assert!(!settings.setting());
    }

    #[test]
    fn setting_accessor_returns_field_value() {
        let settings = ApplicationSettings { setting: true };
        assert!(settings.setting());
    }

    #[test]
    fn clone_produces_equal_value() {
        let settings = ApplicationSettings { setting: true };
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
        let settings = ApplicationSettings { setting: true };
        let json = serde_json::to_string(&settings).expect("serialize ApplicationSettings");
        let deserialized: ApplicationSettings =
            serde_json::from_str(&json).expect("deserialize ApplicationSettings");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn deserialize_missing_field_fails() {
        let result: Result<ApplicationSettings, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }
}
