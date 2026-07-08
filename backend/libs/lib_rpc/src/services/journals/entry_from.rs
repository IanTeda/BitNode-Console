//! Conversion from [`lib_journals::Entry`] to the protobuf [`JournalsEntry`] message.

use crate::generated_protos::journals::{JournalsEntry, Priority};

/// Converts a domain [`lib_journals::Entry`] into the protobuf wire type.
///
/// `priority` is mapped through [`Priority::from`] then cast to `i32` as required
/// by prost's enum representation.  All other fields are moved directly; the
/// `extra_fields` [`BTreeMap`] is collected into the [`HashMap`] expected by prost.
impl From<lib_journals::Entry> for JournalsEntry {
    fn from(e: lib_journals::Entry) -> Self {
        Self {
            message: e.message,
            timestamp_us: e.timestamp_us,
            priority: Priority::from(e.priority) as i32,
            unit: e.unit,
            cursor: e.cursor,
            extra_fields: e.extra_fields.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_entry(
        message: &str,
        timestamp_us: i64,
        priority: lib_journals::Priority,
        unit: &str,
        cursor: Option<&str>,
        extra: BTreeMap<String, String>,
    ) -> lib_journals::Entry {
        lib_journals::Entry {
            message: message.to_owned(),
            timestamp_us,
            priority,
            unit: unit.to_owned(),
            cursor: cursor.map(str::to_owned),
            extra_fields: extra,
        }
    }

    #[test]
    fn maps_all_fields() {
        let mut extra = BTreeMap::new();
        extra.insert("_PID".to_owned(), "1234".to_owned());

        let entry = make_entry(
            "hello world",
            1_000_000,
            lib_journals::Priority::Info,
            "bitcoind",
            Some("s=abc123"),
            extra,
        );

        let proto = JournalsEntry::from(entry);

        assert_eq!(proto.message, "hello world");
        assert_eq!(proto.timestamp_us, 1_000_000);
        assert_eq!(proto.priority, Priority::Info as i32);
        assert_eq!(proto.unit, "bitcoind");
        assert_eq!(proto.cursor.as_deref(), Some("s=abc123"));
        assert_eq!(proto.extra_fields.get("_PID").map(String::as_str), Some("1234"));
    }

    #[test]
    fn absent_cursor_stays_none() {
        let entry = make_entry("msg", 0, lib_journals::Priority::Debug, "", None, BTreeMap::new());
        let proto = JournalsEntry::from(entry);
        assert!(proto.cursor.is_none());
    }

    #[test]
    fn empty_extra_fields() {
        let entry = make_entry("msg", 0, lib_journals::Priority::Info, "", None, BTreeMap::new());
        let proto = JournalsEntry::from(entry);
        assert!(proto.extra_fields.is_empty());
    }

    #[test]
    fn priority_variants_round_trip() {
        use lib_journals::Priority as D;

        let cases = [
            (D::Emergency, Priority::Emergency),
            (D::Alert, Priority::Alert),
            (D::Critical, Priority::Critical),
            (D::Error, Priority::Error),
            (D::Warning, Priority::Warning),
            (D::Notice, Priority::Notice),
            (D::Info, Priority::Info),
            (D::Debug, Priority::Debug),
        ];

        for (domain, expected_proto) in cases {
            let entry = make_entry("", 0, domain, "", None, BTreeMap::new());
            let proto = JournalsEntry::from(entry);
            assert_eq!(proto.priority, expected_proto as i32);
        }
    }
}
