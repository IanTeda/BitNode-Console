//! Journal entry priority levels.
//!
//! Provides the [`Priority`] enum, which maps the numeric `PRIORITY` field
//! stored in the systemd journal to the eight syslog severity levels defined
//! by RFC 5424. Conversions from both `u8` and `&str` are provided so that
//! raw journal field values can be decoded directly into a typed variant.

/// Systemd journal priority levels, matching syslog severity (RFC 5424).
///
/// Variants are ordered from most to least severe, so comparisons such as
/// `priority <= Priority::Error` work as expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// System is unusable (syslog level 0, `emerg`).
    Emergency,

    /// Action must be taken immediately (syslog level 1, `alert`).
    Alert,

    /// Critical conditions (syslog level 2, `crit`).
    Critical,

    /// Error conditions (syslog level 3, `err`).
    Error,

    /// Warning conditions (syslog level 4, `warning`).
    Warning,

    /// Normal but significant conditions (syslog level 5, `notice`).
    Notice,

    /// Informational messages (syslog level 6, `info`).
    Info,

    /// Debug-level messages (syslog level 7, `debug`).
    Debug,
}

impl Priority {
    /// Returns the lowercase syslog name for this priority level.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Emergency => "emerg",
            Self::Alert => "alert",
            Self::Critical => "crit",
            Self::Error => "err",
            Self::Warning => "warning",
            Self::Notice => "notice",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

/// Converts from a numeric priority value (0–7). Values outside this range map to `Info`.
impl From<u8> for Priority {
    fn from(n: u8) -> Self {
        match n {
            0 => Self::Emergency,
            1 => Self::Alert,
            2 => Self::Critical,
            3 => Self::Error,
            4 => Self::Warning,
            5 => Self::Notice,
            6 => Self::Info,
            7 => Self::Debug,
            _ => Self::Info,
        }
    }
}

/// Converts from the numeric string stored in the journal `PRIORITY` field (`"0"`–`"7"`).
/// Unrecognised values (non-numeric or out of range) map to `Info`.
impl From<&str> for Priority {
    fn from(s: &str) -> Self {
        s.parse::<u8>().map(Self::from).unwrap_or(Self::Info)
    }
}

/// Formats the priority using its lowercase syslog name (delegates to [`Priority::as_str`]).
impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8_all_known_values() {
        assert_eq!(Priority::from(0u8), Priority::Emergency);
        assert_eq!(Priority::from(1u8), Priority::Alert);
        assert_eq!(Priority::from(2u8), Priority::Critical);
        assert_eq!(Priority::from(3u8), Priority::Error);
        assert_eq!(Priority::from(4u8), Priority::Warning);
        assert_eq!(Priority::from(5u8), Priority::Notice);
        assert_eq!(Priority::from(6u8), Priority::Info);
        assert_eq!(Priority::from(7u8), Priority::Debug);
    }

    #[test]
    fn from_u8_out_of_range_defaults_to_info() {
        assert_eq!(Priority::from(8u8), Priority::Info);
        assert_eq!(Priority::from(255u8), Priority::Info);
    }

    #[test]
    fn from_str_numeric() {
        assert_eq!(Priority::from("0"), Priority::Emergency);
        assert_eq!(Priority::from("3"), Priority::Error);
        assert_eq!(Priority::from("6"), Priority::Info);
        assert_eq!(Priority::from("7"), Priority::Debug);
    }

    #[test]
    fn from_str_unrecognised_defaults_to_info() {
        assert_eq!(Priority::from(""), Priority::Info);
        assert_eq!(Priority::from("abc"), Priority::Info);
        assert_eq!(Priority::from("8"), Priority::Info);
        assert_eq!(Priority::from("-1"), Priority::Info);
    }

    #[test]
    fn display_uses_syslog_names() {
        assert_eq!(Priority::Emergency.to_string(), "emerg");
        assert_eq!(Priority::Critical.to_string(), "crit");
        assert_eq!(Priority::Error.to_string(), "err");
        assert_eq!(Priority::Info.to_string(), "info");
        assert_eq!(Priority::Debug.to_string(), "debug");
    }
}
