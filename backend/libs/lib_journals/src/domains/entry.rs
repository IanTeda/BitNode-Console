//! Journal entry type.

use std::collections::BTreeMap;

use super::priority::JournalPriority;

/// A single parsed journal entry.
#[derive(Debug)]
pub struct JournalEntry {
    /// The log message.
    pub message: String,

    /// Timestamp in microseconds since Unix epoch (from `journal.timestamp_usec()`).
    pub timestamp_us: i64,

    /// Syslog priority level.
    pub priority: JournalPriority,

    /// Unit identifier: `SYSLOG_IDENTIFIER` if present, else `_SYSTEMD_UNIT`, else empty.
    ///
    /// `SYSLOG_IDENTIFIER` is preferred because it reflects what the process names itself.
    /// `_SYSTEMD_UNIT` is often a generic parent (e.g. `user@1200.service`) when a daemon
    /// runs directly in a user session rather than as a dedicated systemd unit.
    pub unit: String,

    /// Journal cursor string, used for seeking and pagination.
    pub cursor: Option<String>,

    /// All remaining fields from the journal record not captured by the fields above.
    pub extra_fields: BTreeMap<String, String>,
}

impl JournalEntry {
    /// Build a `JournalEntry` from a raw journal record and its timestamp.
    ///
    /// `timestamp_us` comes from `journal.timestamp_usec()` rather than the record itself
    /// because it is sourced from a separate libsystemd call on the current cursor position.
    #[must_use]
    pub fn from_record(record: &BTreeMap<String, String>, timestamp_us: i64) -> Self {
        const KNOWN: &[&str] = &[
            "MESSAGE",
            "PRIORITY",
            "_SYSTEMD_UNIT",
            "SYSLOG_IDENTIFIER",
            "__CURSOR",
        ];

        let message = record.get("MESSAGE").cloned().unwrap_or_default();
        let priority = record
            .get("PRIORITY")
            .map_or(JournalPriority::Info, |s| JournalPriority::from(s.as_str()));
        let unit = record
            .get("SYSLOG_IDENTIFIER")
            .or_else(|| record.get("_SYSTEMD_UNIT"))
            .cloned()
            .unwrap_or_default();
        let cursor = record.get("__CURSOR").cloned();
        let extra_fields = record
            .iter()
            .filter(|(k, _)| !KNOWN.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Self { message, timestamp_us, priority, unit, cursor, extra_fields }
    }
}

#[cfg(test)]
mod tests {
    use super::JournalEntry;
    use super::super::priority::JournalPriority;
    use std::collections::BTreeMap;

    #[test]
    fn prefers_syslog_identifier_over_systemd_unit() {
        let mut record = BTreeMap::new();
        record.insert("MESSAGE".into(), "hello".into());
        record.insert("PRIORITY".into(), "6".into());
        record.insert("_SYSTEMD_UNIT".into(), "user@1200.service".into());
        record.insert("SYSLOG_IDENTIFIER".into(), "bitcoind".into());
        record.insert("__CURSOR".into(), "s=abc123".into());
        record.insert("_PID".into(), "1234".into());

        let entry = JournalEntry::from_record(&record, 1_000_000);

        assert_eq!(entry.message, "hello");
        assert_eq!(entry.priority, JournalPriority::Info);
        assert_eq!(entry.unit, "bitcoind");
        assert_eq!(entry.cursor.as_deref(), Some("s=abc123"));
        assert_eq!(entry.timestamp_us, 1_000_000);
        assert!(entry.extra_fields.contains_key("_PID"));
        assert!(!entry.extra_fields.contains_key("MESSAGE"));
        assert!(!entry.extra_fields.contains_key("__CURSOR"));
    }

    #[test]
    fn falls_back_to_systemd_unit_when_no_syslog_identifier() {
        let mut record = BTreeMap::new();
        record.insert("MESSAGE".into(), "hello".into());
        record.insert("_SYSTEMD_UNIT".into(), "bitcoind.service".into());

        let entry = JournalEntry::from_record(&record, 0);

        assert_eq!(entry.unit, "bitcoind.service");
    }

    #[test]
    fn falls_back_to_syslog_identifier() {
        let mut record = BTreeMap::new();
        record.insert("MESSAGE".into(), "world".into());
        record.insert("PRIORITY".into(), "3".into());
        record.insert("SYSLOG_IDENTIFIER".into(), "bitcoind".into());

        let entry = JournalEntry::from_record(&record, 0);

        assert_eq!(entry.unit, "bitcoind");
        assert_eq!(entry.priority, JournalPriority::Error);
        assert!(entry.cursor.is_none());
    }

    #[test]
    fn missing_optional_fields_uses_defaults() {
        let record = BTreeMap::new();

        let entry = JournalEntry::from_record(&record, 0);

        assert_eq!(entry.message, "");
        assert_eq!(entry.priority, JournalPriority::Info);
        assert_eq!(entry.unit, "");
        assert!(entry.cursor.is_none());
        assert!(entry.extra_fields.is_empty());
    }
}
