//! Conversion from [`lib_journals::Page`] into [`GetJournalsResponse`].
//!
//! Each [`lib_journals::Entry`] in the page is mapped element-by-element to a
//! [`JournalsEntry`] proto message via the [`From`] impl in `entry_from`.  The
//! domain [`PaginationResponse`] (next/prev cursor tokens) is converted to a
//! proto [`PageResponse`] and always wrapped in `Some`, so
//! [`GetJournalsResponse::pagination`] is never absent in the wire response.

use crate::generated_protos::journals::{GetJournalsResponse, JournalsEntry};
use crate::services::pagination::PageResponse;

/// Converts a [`lib_journals::Page`] into the protobuf [`GetJournalsResponse`].
///
/// Entry order is preserved.  Pagination cursors are forwarded verbatim from
/// the domain [`PaginationResponse`] to the proto [`PageResponse`]; absent
/// tokens remain `None` in the proto.
impl From<lib_journals::Page> for GetJournalsResponse {
    fn from(page: lib_journals::Page) -> Self {
        Self {
            entries: page.entries.into_iter().map(JournalsEntry::from).collect(),
            pagination: Some(PageResponse::from(page.pagination)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lib_journals::{Entry, Page, Priority};

    use super::*;
    use crate::pagination_from::PaginationResponse;

    fn make_entry(message: &str) -> Entry {
        Entry {
            message: message.to_owned(),
            timestamp_us: 0,
            priority: Priority::Info,
            unit: String::new(),
            cursor: None,
            extra_fields: BTreeMap::new(),
        }
    }

    fn make_page(
        entries: Vec<Entry>,
        next_token: Option<&str>,
        prev_token: Option<&str>,
    ) -> Page {
        Page {
            entries,
            pagination: PaginationResponse {
                next_page_token: next_token.map(str::to_owned),
                prev_page_token: prev_token.map(str::to_owned),
            },
        }
    }

    #[test]
    fn empty_page_produces_empty_entries() {
        let response = GetJournalsResponse::from(make_page(vec![], None, None));
        assert!(response.entries.is_empty());
    }

    /// `pagination` is always `Some` — the proto field is optional by type but
    /// the conversion always sets it.
    #[test]
    fn pagination_is_always_some() {
        let response = GetJournalsResponse::from(make_page(vec![], None, None));
        assert!(response.pagination.is_some());
    }

    #[test]
    fn entry_count_matches_page() {
        let page = make_page(
            vec![make_entry("a"), make_entry("b"), make_entry("c")],
            None,
            None,
        );
        assert_eq!(GetJournalsResponse::from(page).entries.len(), 3);
    }

    /// Entries must appear in the same order they were in the page so callers
    /// receive log lines in chronological sequence.
    #[test]
    fn entry_order_is_preserved() {
        let page = make_page(vec![make_entry("first"), make_entry("second")], None, None);
        let entries = GetJournalsResponse::from(page).entries;
        assert_eq!(entries[0].message, "first");
        assert_eq!(entries[1].message, "second");
    }

    #[test]
    fn next_page_token_is_forwarded() {
        let page = make_page(vec![], Some("cursor-next"), None);
        let pagination = GetJournalsResponse::from(page).pagination.unwrap();
        assert_eq!(pagination.page_token_next.as_deref(), Some("cursor-next"));
    }

    #[test]
    fn prev_page_token_is_forwarded() {
        let page = make_page(vec![], None, Some("cursor-prev"));
        let pagination = GetJournalsResponse::from(page).pagination.unwrap();
        assert_eq!(pagination.page_token_prev.as_deref(), Some("cursor-prev"));
    }

    #[test]
    fn absent_tokens_remain_none() {
        let pagination = GetJournalsResponse::from(make_page(vec![], None, None))
            .pagination
            .unwrap();
        assert!(pagination.page_token_next.is_none());
        assert!(pagination.page_token_prev.is_none());
    }
}
