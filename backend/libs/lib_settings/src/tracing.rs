//-- ./backend/libs/lib_settings/src/tracing.rs

//! # Tracing Configuration
//!
//! This module provides configuration structures for the tracing system.
//!
//! The tracing configuration allows users to customize logging verbosity and behaviour through
//! configuration files, environment variables, or programmatic settings.

/// Default enabled state if none is provided.
const DEFAULT_ENABLED: bool = true;

/// Default telemetry level if none is provided.
const DEFAULT_TELEMETRY_LEVEL: lib_tracing::TracingLevels = lib_tracing::TracingLevels::INFO;

/// Default show settings startup state if none is provided.
const DEFAULT_SHOW_SETTINGS_STARTUP: bool = false;

/// Settings struct for the tracing library crate.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct TracingSettings {
    /// Whether to enable telemetry logging.
    pub enabled: bool,

    /// The telemetry level to use for logging.
    pub level: lib_tracing::TracingLevels,

    /// Whether to show settings startup information.
    pub show_settings_startup: bool,
}

impl Default for TracingSettings {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_ENABLED,
            level: DEFAULT_TELEMETRY_LEVEL,
            show_settings_startup: DEFAULT_SHOW_SETTINGS_STARTUP,
        }
    }
}

impl TracingSettings {
    /// Returns the configured telemetry level.
    #[must_use]
    pub const fn telemetry_level(&self) -> lib_tracing::TracingLevels {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_tracing::TracingLevels;

    #[test]
    fn default_enables_tracing_at_info_level() {
        let settings = TracingSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.level, TracingLevels::INFO);
        assert!(!settings.show_settings_startup);
    }

    #[test]
    fn telemetry_level_returns_configured_level() {
        let settings = TracingSettings {
            enabled: true,
            level: TracingLevels::DEBUG,
            show_settings_startup: false,
        };
        assert_eq!(settings.telemetry_level(), TracingLevels::DEBUG);
    }

    #[test]
    fn telemetry_level_returns_correct_value_for_all_variants() {
        for level in [
            TracingLevels::OFF,
            TracingLevels::ERROR,
            TracingLevels::WARN,
            TracingLevels::INFO,
            TracingLevels::DEBUG,
            TracingLevels::TRACE,
        ] {
            let settings = TracingSettings {
                enabled: true,
                level,
                show_settings_startup: false,
            };
            assert_eq!(settings.telemetry_level(), level);
        }
    }

    #[test]
    fn clone_produces_equal_value() {
        let settings = TracingSettings {
            enabled: false,
            level: TracingLevels::TRACE,
            show_settings_startup: true,
        };
        assert_eq!(settings.clone(), settings);
    }

    #[test]
    fn equal_settings_compare_equal() {
        let a = TracingSettings::default();
        let b = TracingSettings::default();
        assert_eq!(a, b);
    }

    #[test]
    fn different_levels_compare_unequal() {
        let a = TracingSettings {
            level: TracingLevels::DEBUG,
            ..Default::default()
        };
        let b = TracingSettings {
            level: TracingLevels::ERROR,
            ..Default::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn different_enabled_flags_compare_unequal() {
        let a = TracingSettings {
            enabled: true,
            ..Default::default()
        };
        let b = TracingSettings {
            enabled: false,
            ..Default::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn different_show_settings_flags_compare_unequal() {
        let a = TracingSettings {
            show_settings_startup: false,
            ..Default::default()
        };
        let b = TracingSettings {
            show_settings_startup: true,
            ..Default::default()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn debug_format_includes_struct_name_and_level() {
        let settings = TracingSettings::default();
        let debug_str = format!("{settings:?}");
        assert!(debug_str.contains("TracingSettings"));
        assert!(debug_str.contains("INFO"));
    }

    #[test]
    fn serialize_to_json_roundtrip() {
        let settings = TracingSettings {
            enabled: true,
            level: TracingLevels::WARN,
            show_settings_startup: true,
        };
        let json = serde_json::to_string(&settings).expect("serialize TracingSettings");
        let deserialized: TracingSettings =
            serde_json::from_str(&json).expect("deserialize TracingSettings");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn serialize_level_as_lowercase_string() {
        let settings = TracingSettings {
            level: TracingLevels::DEBUG,
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).expect("serialize TracingSettings");
        assert!(
            json.contains("\"debug\""),
            "level should serialize as lowercase: {json}"
        );
    }

    #[test]
    fn deserialize_from_explicit_json_values() {
        let json = r#"{"enabled": false, "level": "trace", "show_settings_startup": true}"#;
        let settings: TracingSettings =
            serde_json::from_str(json).expect("deserialize TracingSettings");
        assert!(!settings.enabled);
        assert_eq!(settings.level, TracingLevels::TRACE);
        assert!(settings.show_settings_startup);
    }

    #[test]
    fn deserialize_missing_field_fails() {
        let result: Result<TracingSettings, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_invalid_level_string() {
        let json = r#"{"enabled": true, "level": "verbose", "show_settings_startup": false}"#;
        let result: Result<TracingSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_uppercase_level_string() {
        let json = r#"{"enabled": true, "level": "INFO", "show_settings_startup": false}"#;
        let result: Result<TracingSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_produces_expected_json_field_names() {
        let settings = TracingSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize TracingSettings");
        assert!(json.contains("\"enabled\""), "missing 'enabled' field: {json}");
        assert!(json.contains("\"level\""), "missing 'level' field: {json}");
        assert!(
            json.contains("\"show_settings_startup\""),
            "missing 'show_settings_startup' field: {json}"
        );
    }

    #[test]
    fn deserialize_all_level_variants() {
        for (json_str, expected) in [
            ("\"off\"", TracingLevels::OFF),
            ("\"error\"", TracingLevels::ERROR),
            ("\"warn\"", TracingLevels::WARN),
            ("\"info\"", TracingLevels::INFO),
            ("\"debug\"", TracingLevels::DEBUG),
            ("\"trace\"", TracingLevels::TRACE),
        ] {
            let json =
                format!(r#"{{"enabled": true, "level": {json_str}, "show_settings_startup": false}}"#);
            let settings: TracingSettings =
                serde_json::from_str(&json).unwrap_or_else(|_| panic!("failed to deserialize level {json_str}"));
            assert_eq!(settings.level, expected);
        }
    }
}
