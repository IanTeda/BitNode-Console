//! Journal page type.
//!
//! A [`Page`] bundles a slice of [`Entry`] values returned by a single seek
//! operation with the cursor-based [`PaginationResponse`] metadata needed to
//! request the next or previous page.

use crate::domains;

/// A page of journal entries returned by a seek operation.
#[derive(Debug)]
pub struct Page {
    /// Entries in ascending timestamp order.
    pub entries: Vec<domains::Entry>,

    /// Cursor-based pagination metadata for this page.
    pub pagination: lib_core::domains::pagination::PaginationResponse,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lib_core::domains::pagination::PaginationResponse;

    use crate::{Entry, Page};

    fn make_entry(message: &str, timestamp_us: i64) -> Entry {
        let mut record = BTreeMap::new();
        record.insert("MESSAGE".into(), message.into());
        Entry::from_record(&record, timestamp_us)
    }

    #[test]
    fn empty_page_has_no_entries_and_no_tokens() {
        let page = Page {
            entries: vec![],
            pagination: PaginationResponse::default(),
        };

        assert!(page.entries.is_empty());
        assert!(page.pagination.next_page_token.is_none());
        assert!(page.pagination.prev_page_token.is_none());
    }

    #[test]
    fn page_stores_entries_in_insertion_order() {
        let entries = vec![
            make_entry("first", 1_000),
            make_entry("second", 2_000),
            make_entry("third", 3_000),
        ];

        let page = Page {
            entries,
            pagination: PaginationResponse::default(),
        };

        assert_eq!(page.entries.len(), 3);
        assert_eq!(page.entries[0].message, "first");
        assert_eq!(page.entries[1].message, "second");
        assert_eq!(page.entries[2].message, "third");
    }

    #[test]
    fn page_exposes_pagination_tokens() {
        let page = Page {
            entries: vec![make_entry("msg", 1_000)],
            pagination: PaginationResponse {
                next_page_token: Some("cursor-next".into()),
                prev_page_token: Some("cursor-prev".into()),
            },
        };

        assert_eq!(
            page.pagination.next_page_token.as_deref(),
            Some("cursor-next")
        );
        assert_eq!(
            page.pagination.prev_page_token.as_deref(),
            Some("cursor-prev")
        );
    }
}
