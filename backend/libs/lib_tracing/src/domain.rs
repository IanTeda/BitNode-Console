//-- ./backend/libs/lib_tracing/src/domain.rs

//! Tracing domain types.

/// Tracing levels [OFF, ERROR, WARN, INFO, DEBUG, TRACE].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TracingLevels {
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

/// Conversion from `TracingLevels` to `tracing::LevelFilter`.
///
/// This implementation allows seamless integration with the tracing ecosystem,
/// enabling configuration-driven telemetry level control throughout the application.
///
/// The conversion is infallible and maintains the same semantic meaning for each level.
impl From<TracingLevels> for tracing::level_filters::LevelFilter {
    fn from(level: TracingLevels) -> Self {
        match level {
            TracingLevels::OFF => Self::OFF,
            TracingLevels::ERROR => Self::ERROR,
            TracingLevels::WARN => Self::WARN,
            TracingLevels::INFO => Self::INFO,
            TracingLevels::DEBUG => Self::DEBUG,
            TracingLevels::TRACE => Self::TRACE,
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
impl std::fmt::Display for TracingLevels {
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
        assert_eq!(TracingLevels::default(), TracingLevels::WARN);
    }

    #[test]
    fn display_matches_serde_lowercase_names() {
        assert_eq!(TracingLevels::OFF.to_string(), "off");
        assert_eq!(TracingLevels::ERROR.to_string(), "error");
        assert_eq!(TracingLevels::WARN.to_string(), "warn");
        assert_eq!(TracingLevels::INFO.to_string(), "info");
        assert_eq!(TracingLevels::DEBUG.to_string(), "debug");
        assert_eq!(TracingLevels::TRACE.to_string(), "trace");
    }

    #[test]
    fn converts_to_level_filter() {
        use tracing::level_filters::LevelFilter;

        assert_eq!(LevelFilter::from(TracingLevels::OFF), LevelFilter::OFF);
        assert_eq!(LevelFilter::from(TracingLevels::ERROR), LevelFilter::ERROR);
        assert_eq!(LevelFilter::from(TracingLevels::WARN), LevelFilter::WARN);
        assert_eq!(LevelFilter::from(TracingLevels::INFO), LevelFilter::INFO);
        assert_eq!(LevelFilter::from(TracingLevels::DEBUG), LevelFilter::DEBUG);
        assert_eq!(LevelFilter::from(TracingLevels::TRACE), LevelFilter::TRACE);
    }

    #[test]
    fn serializes_to_lowercase_json_strings() {
        assert_eq!(
            serde_json::to_string(&TracingLevels::OFF).unwrap(),
            "\"off\""
        );
        assert_eq!(
            serde_json::to_string(&TracingLevels::ERROR).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&TracingLevels::WARN).unwrap(),
            "\"warn\""
        );
        assert_eq!(
            serde_json::to_string(&TracingLevels::INFO).unwrap(),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&TracingLevels::DEBUG).unwrap(),
            "\"debug\""
        );
        assert_eq!(
            serde_json::to_string(&TracingLevels::TRACE).unwrap(),
            "\"trace\""
        );
    }

    #[test]
    fn deserializes_from_lowercase_json_strings() {
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"off\"").unwrap(),
            TracingLevels::OFF
        );
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"error\"").unwrap(),
            TracingLevels::ERROR
        );
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"warn\"").unwrap(),
            TracingLevels::WARN
        );
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"info\"").unwrap(),
            TracingLevels::INFO
        );
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"debug\"").unwrap(),
            TracingLevels::DEBUG
        );
        assert_eq!(
            serde_json::from_str::<TracingLevels>("\"trace\"").unwrap(),
            TracingLevels::TRACE
        );
    }

    #[test]
    fn deserialize_rejects_unknown_variant() {
        let result = serde_json::from_str::<TracingLevels>("\"verbose\"");

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_uppercase_variant() {
        let result = serde_json::from_str::<TracingLevels>("\"INFO\"");

        assert!(result.is_err());
    }

    #[test]
    fn is_clone_and_copy() {
        let level = TracingLevels::DEBUG;
        let cloned = Clone::clone(&level);
        let copied = level;

        assert_eq!(level, cloned);
        assert_eq!(level, copied);
    }

    #[test]
    fn debug_format_includes_variant_name() {
        assert_eq!(format!("{:?}", TracingLevels::TRACE), "TRACE");
    }
}
