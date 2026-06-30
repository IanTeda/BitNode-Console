//! Journal priority level.

/// Systemd journal priority levels, matching syslog severity (RFC 5424).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JournalPriority {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl JournalPriority {
    /// Returns the lowercase syslog name for this priority level.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Emergency => "emerg",
            Self::Alert     => "alert",
            Self::Critical  => "crit",
            Self::Error     => "err",
            Self::Warning   => "warning",
            Self::Notice    => "notice",
            Self::Info      => "info",
            Self::Debug     => "debug",
        }
    }
}

/// Converts from a numeric priority value (0–7). Values outside this range map to `Info`.
impl From<u8> for JournalPriority {
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
impl From<&str> for JournalPriority {
    fn from(s: &str) -> Self {
        s.parse::<u8>().map(Self::from).unwrap_or(Self::Info)
    }
}

impl std::fmt::Display for JournalPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8_all_known_values() {
        assert_eq!(JournalPriority::from(0u8), JournalPriority::Emergency);
        assert_eq!(JournalPriority::from(1u8), JournalPriority::Alert);
        assert_eq!(JournalPriority::from(2u8), JournalPriority::Critical);
        assert_eq!(JournalPriority::from(3u8), JournalPriority::Error);
        assert_eq!(JournalPriority::from(4u8), JournalPriority::Warning);
        assert_eq!(JournalPriority::from(5u8), JournalPriority::Notice);
        assert_eq!(JournalPriority::from(6u8), JournalPriority::Info);
        assert_eq!(JournalPriority::from(7u8), JournalPriority::Debug);
    }

    #[test]
    fn from_u8_out_of_range_defaults_to_info() {
        assert_eq!(JournalPriority::from(8u8), JournalPriority::Info);
        assert_eq!(JournalPriority::from(255u8), JournalPriority::Info);
    }

    #[test]
    fn from_str_numeric() {
        assert_eq!(JournalPriority::from("0"), JournalPriority::Emergency);
        assert_eq!(JournalPriority::from("3"), JournalPriority::Error);
        assert_eq!(JournalPriority::from("6"), JournalPriority::Info);
        assert_eq!(JournalPriority::from("7"), JournalPriority::Debug);
    }

    #[test]
    fn from_str_unrecognised_defaults_to_info() {
        assert_eq!(JournalPriority::from(""), JournalPriority::Info);
        assert_eq!(JournalPriority::from("abc"), JournalPriority::Info);
        assert_eq!(JournalPriority::from("8"), JournalPriority::Info);
        assert_eq!(JournalPriority::from("-1"), JournalPriority::Info);
    }

    #[test]
    fn display_uses_syslog_names() {
        assert_eq!(JournalPriority::Emergency.to_string(), "emerg");
        assert_eq!(JournalPriority::Critical.to_string(), "crit");
        assert_eq!(JournalPriority::Error.to_string(), "err");
        assert_eq!(JournalPriority::Info.to_string(), "info");
        assert_eq!(JournalPriority::Debug.to_string(), "debug");
    }
}
