//-- ./backend/libs/lib_tracing/src/domain.rs

//! Telemetry domain types.

/// Telemetry levels [OFF, ERROR, WARN, INFO, DEBUG, TRACE].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryLevels {
    /// No telemetry output.
    ///
    /// Completely disables all telemetry output. Useful for performance-critical
    /// environments or when telemetry is not needed.
    OFF,

    /// Error-level telemetry only.
    ///
    /// Telemetry logs only error conditions that may require attention. This is the most
    /// minimal telemetry level for production systems.
    ERROR,

    /// Warning and error-level telemetry.
    ///
    /// Telemetry logs warnings and errors. This is the default level, providing visibility
    /// into potential issues while minimising telemetry volume.
    #[default]
    WARN,

    /// Informational, warning, and error-level telemetry.
    ///
    /// Telemetry logs informational messages, warnings, and errors. Useful for understanding
    /// application flow and identifying potential issues.
    INFO,

    /// Debug, informational, warning, and error-level telemetry.
    ///
    /// Includes debug information for troubleshooting. Suitable for development
    /// and staging environments.
    DEBUG,

    /// All telemetry levels including trace information.
    ///
    /// Maximum verbosity including trace-level information. Primarily used for
    /// detailed debugging and development. May impact performance due to high
    /// telemetry volume.
    TRACE,
}

/// Conversion from `TelemetryLevels` to `tracing::LevelFilter`.
///
/// This implementation allows seamless integration with the tracing ecosystem,
/// enabling configuration-driven telemetry level control throughout the application.
///
/// The conversion is infallible and maintains the same semantic meaning for each level.
impl From<TelemetryLevels> for tracing::level_filters::LevelFilter {
    fn from(level: TelemetryLevels) -> Self {
        match level {
            TelemetryLevels::OFF => Self::OFF,
            TelemetryLevels::ERROR => Self::ERROR,
            TelemetryLevels::WARN => Self::WARN,
            TelemetryLevels::INFO => Self::INFO,
            TelemetryLevels::DEBUG => Self::DEBUG,
            TelemetryLevels::TRACE => Self::TRACE,
        }
    }
}

/// Formats the telemetry level as a lowercase string.
///
/// This implementation matches the serde serialization format, producing
/// lowercase strings like "info", "debug", etc. This ensures consistency
/// between serialized configuration and string representations.
///
/// # Examples
///
/// ```rust
/// use lib_tracing::TelemetryLevels;
///
/// assert_eq!(format!("{}", TelemetryLevels::INFO), "info");
/// assert_eq!(format!("{}", TelemetryLevels::DEBUG), "debug");
/// ```
impl std::fmt::Display for TelemetryLevels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self {
            Self::OFF => "off",
            Self::ERROR => "error",
            Self::WARN => "warn",
            Self::INFO => "info",
            Self::DEBUG => "debug",
            Self::TRACE => "trace",
        };
        write!(f, "{level_str}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_warn() {
        assert_eq!(TelemetryLevels::default(), TelemetryLevels::WARN);
    }

    #[test]
    fn display_matches_serde_lowercase_names() {
        assert_eq!(TelemetryLevels::OFF.to_string(), "off");
        assert_eq!(TelemetryLevels::ERROR.to_string(), "error");
        assert_eq!(TelemetryLevels::WARN.to_string(), "warn");
        assert_eq!(TelemetryLevels::INFO.to_string(), "info");
        assert_eq!(TelemetryLevels::DEBUG.to_string(), "debug");
        assert_eq!(TelemetryLevels::TRACE.to_string(), "trace");
    }

    #[test]
    fn converts_to_level_filter() {
        use tracing::level_filters::LevelFilter;

        assert_eq!(LevelFilter::from(TelemetryLevels::OFF), LevelFilter::OFF);
        assert_eq!(LevelFilter::from(TelemetryLevels::ERROR), LevelFilter::ERROR);
        assert_eq!(LevelFilter::from(TelemetryLevels::WARN), LevelFilter::WARN);
        assert_eq!(LevelFilter::from(TelemetryLevels::INFO), LevelFilter::INFO);
        assert_eq!(LevelFilter::from(TelemetryLevels::DEBUG), LevelFilter::DEBUG);
        assert_eq!(LevelFilter::from(TelemetryLevels::TRACE), LevelFilter::TRACE);
    }

    #[test]
    fn serializes_to_lowercase_json_strings() {
        assert_eq!(serde_json::to_string(&TelemetryLevels::OFF).unwrap(), "\"off\"");
        assert_eq!(serde_json::to_string(&TelemetryLevels::ERROR).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&TelemetryLevels::WARN).unwrap(), "\"warn\"");
        assert_eq!(serde_json::to_string(&TelemetryLevels::INFO).unwrap(), "\"info\"");
        assert_eq!(serde_json::to_string(&TelemetryLevels::DEBUG).unwrap(), "\"debug\"");
        assert_eq!(serde_json::to_string(&TelemetryLevels::TRACE).unwrap(), "\"trace\"");
    }

    #[test]
    fn deserializes_from_lowercase_json_strings() {
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"off\"").unwrap(),
            TelemetryLevels::OFF
        );
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"error\"").unwrap(),
            TelemetryLevels::ERROR
        );
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"warn\"").unwrap(),
            TelemetryLevels::WARN
        );
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"info\"").unwrap(),
            TelemetryLevels::INFO
        );
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"debug\"").unwrap(),
            TelemetryLevels::DEBUG
        );
        assert_eq!(
            serde_json::from_str::<TelemetryLevels>("\"trace\"").unwrap(),
            TelemetryLevels::TRACE
        );
    }

    #[test]
    fn deserialize_rejects_unknown_variant() {
        let result = serde_json::from_str::<TelemetryLevels>("\"verbose\"");

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_uppercase_variant() {
        let result = serde_json::from_str::<TelemetryLevels>("\"INFO\"");

        assert!(result.is_err());
    }

    #[test]
    fn is_clone_and_copy() {
        let level = TelemetryLevels::DEBUG;
        let cloned = Clone::clone(&level);
        let copied = level;

        assert_eq!(level, cloned);
        assert_eq!(level, copied);
    }

    #[test]
    fn debug_format_includes_variant_name() {
        assert_eq!(format!("{:?}", TelemetryLevels::TRACE), "TRACE");
    }
}
