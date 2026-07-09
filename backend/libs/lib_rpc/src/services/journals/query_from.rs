//! Conversion from [`GetJournalsRequest`] into [`lib_journals::Query`].
//!
//! Four mapping rules apply:
//!
//! * **`unit_name`** — not present in the request; the returned struct always
//!   carries an empty-string placeholder.  Callers must inject the real value
//!   (from server configuration) before dispatching.
//! * **`priority`** — `None`, an unrecognised `i32`, or `Unspecified` (the
//!   proto zero-value default) all fall back to [`lib_journals::Priority::Info`].
//!   Named proto variants are mapped 1-to-1 via the [`From`] impl in
//!   `priority_from`.
//! * **`pagination`** — `None` or a `page_size` of `0` fall back to the domain
//!   default via [`PaginationRequest::default`].
//! * **`timestamp_from_us` / `timestamp_to_us`** — forwarded verbatim as
//!   `Option<i64>`; absent means no bound.

use crate::generated_protos::journals::{GetJournalsRequest, Priority};
use crate::pagination_from::{DEFAULT_PAGE_SIZE, PaginationRequest};

/// Converts a gRPC [`GetJournalsRequest`] into a domain [`lib_journals::Query`].
///
/// `unit_name` is not part of the request — it is fixed by server
/// configuration.  The returned query carries an empty-string placeholder;
/// callers must inject the real value before dispatching.  The pattern used
/// by the `GetJournals` handler is:
///
/// ```rust,ignore
/// let mut query: lib_journals::Query = request.into_inner().into();
/// query.unit_name = unit_name;
/// ```
///
/// The `priority` field goes through two hops: `Option<i32>` →
/// `Option<Priority>` (via [`Priority::try_from`], dropping invalid values)
/// → `lib_journals::Priority` (via [`From<Priority>`] from `priority_from`).
/// Any gap in that chain falls back to `Info`.
impl From<GetJournalsRequest> for lib_journals::Query<'static> {
    fn from(req: GetJournalsRequest) -> Self {
        let priority = req
            .priority
            .and_then(|i| Priority::try_from(i).ok())
            .map(lib_journals::Priority::from)
            .unwrap_or(lib_journals::Priority::Info);

        let pagination = req.pagination.map(PaginationRequest::from).unwrap_or_default();

        Self {
            unit_name: "",
            timestamp_from: req.timestamp_from_us,
            timestamp_to: req.timestamp_to_us,
            priority,
            pagination,
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_journals::{Priority as DomainPriority, Query};

    use super::*;
    use crate::generated_protos::common::PageRequest;

    fn make_req(
        timestamp_from_us: Option<i64>,
        timestamp_to_us: Option<i64>,
        priority: Option<i32>,
        pagination: Option<PageRequest>,
    ) -> GetJournalsRequest {
        GetJournalsRequest {
            timestamp_from_us,
            timestamp_to_us,
            priority,
            pagination,
        }
    }

    fn empty_req() -> GetJournalsRequest {
        make_req(None, None, None, None)
    }

    fn page_req(page_size: u32) -> PageRequest {
        PageRequest {
            page_size,
            page_token: None,
            page_direction: 0,
        }
    }

    // ── unit_name ─────────────────────────────────────────────────────────────

    /// `unit_name` is always the empty-string placeholder — the handler injects
    /// the real value from service configuration after conversion.
    #[test]
    fn unit_name_is_always_empty() {
        assert_eq!(Query::from(empty_req()).unit_name, "");
    }

    // ── priority ──────────────────────────────────────────────────────────────

    #[test]
    fn absent_priority_defaults_to_info() {
        assert_eq!(Query::from(empty_req()).priority, DomainPriority::Info);
    }

    #[test]
    fn invalid_priority_i32_falls_back_to_info() {
        let req = make_req(None, None, Some(999), None);
        assert_eq!(Query::from(req).priority, DomainPriority::Info);
    }

    #[test]
    fn explicit_priority_is_forwarded() {
        let req = make_req(None, None, Some(Priority::Warning as i32), None);
        assert_eq!(Query::from(req).priority, DomainPriority::Warning);
    }

    // ── timestamps ────────────────────────────────────────────────────────────

    #[test]
    fn timestamps_pass_through() {
        let req = make_req(Some(1_000_000), Some(2_000_000), None, None);
        let query = Query::from(req);
        assert_eq!(query.timestamp_from, Some(1_000_000));
        assert_eq!(query.timestamp_to, Some(2_000_000));
    }

    #[test]
    fn absent_timestamps_are_none() {
        let query = Query::from(empty_req());
        assert!(query.timestamp_from.is_none());
        assert!(query.timestamp_to.is_none());
    }

    // ── pagination ────────────────────────────────────────────────────────────

    #[test]
    fn absent_pagination_uses_default() {
        assert_eq!(
            Query::from(empty_req()).pagination,
            PaginationRequest::default()
        );
    }

    #[test]
    fn pagination_page_size_passes_through() {
        let req = make_req(None, None, None, Some(page_req(25)));
        assert_eq!(Query::from(req).pagination.page_size, 25);
    }

    #[test]
    fn zero_page_size_uses_default() {
        let req = make_req(None, None, None, Some(page_req(0)));
        assert_eq!(Query::from(req).pagination.page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_token_passes_through() {
        let req = make_req(
            None,
            None,
            None,
            Some(PageRequest {
                page_size: 10,
                page_token: Some("cursor-abc".to_owned()),
                page_direction: 0,
            }),
        );
        assert_eq!(
            Query::from(req).pagination.page_token.as_deref(),
            Some("cursor-abc"),
        );
    }
}
