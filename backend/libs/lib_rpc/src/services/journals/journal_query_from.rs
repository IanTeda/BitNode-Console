//! Conversion from [`GetJournalsRequest`] into [`JournalQuery`].

use lib_journals::{JournalPriority, JournalQuery};

use crate::generated_protos::journals::{GetJournalsRequest, Priority};
use crate::pagination_from::{DEFAULT_PAGE_SIZE, PaginationRequest};

// ── Priority ──────────────────────────────────────────────────────────────────

impl From<Priority> for JournalPriority {
    fn from(p: Priority) -> Self {
        match p {
            Priority::Unspecified => Self::Info,
            Priority::Emergency   => Self::Emergency,
            Priority::Alert       => Self::Alert,
            Priority::Critical    => Self::Critical,
            Priority::Error       => Self::Error,
            Priority::Warning     => Self::Warning,
            Priority::Notice      => Self::Notice,
            Priority::Info        => Self::Info,
            Priority::Debug       => Self::Debug,
        }
    }
}

// ── GetJournalsRequest → JournalQuery ─────────────────────────────────────────

impl From<GetJournalsRequest> for JournalQuery<'static> {
    /// Converts a gRPC [`GetJournalsRequest`] into a domain [`JournalQuery`].
    ///
    /// `unit_name` is not part of the request — it is fixed by server
    /// configuration. The returned query uses the empty-string default; callers
    /// must supply the real unit name from settings before dispatching, e.g.:
    ///
    /// ```rust,ignore
    /// let query = JournalQuery { unit_name: &settings.unit_name, ..JournalQuery::from(req) };
    /// ```
    fn from(req: GetJournalsRequest) -> Self {
        let priority = req
            .priority
            .and_then(|i| Priority::try_from(i).ok())
            .map(JournalPriority::from)
            .unwrap_or(JournalPriority::Info);

        let pagination = req
            .pagination
            .map(PaginationRequest::from)
            .unwrap_or_default();

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
    use super::*;
    use crate::generated_protos::common::PageRequest;

    // ── Priority ──────────────────────────────────────────────────────────────

    #[test]
    fn priority_unspecified_maps_to_info() {
        assert_eq!(JournalPriority::from(Priority::Unspecified), JournalPriority::Info);
    }

    #[test]
    fn priority_all_named_values() {
        let cases = [
            (Priority::Emergency, JournalPriority::Emergency),
            (Priority::Alert,     JournalPriority::Alert),
            (Priority::Critical,  JournalPriority::Critical),
            (Priority::Error,     JournalPriority::Error),
            (Priority::Warning,   JournalPriority::Warning),
            (Priority::Notice,    JournalPriority::Notice),
            (Priority::Info,      JournalPriority::Info),
            (Priority::Debug,     JournalPriority::Debug),
        ];
        for (proto, domain) in cases {
            assert_eq!(JournalPriority::from(proto), domain);
        }
    }

    // ── GetJournalsRequest ────────────────────────────────────────────────────

    #[test]
    fn timestamps_pass_through() {
        let req = GetJournalsRequest {
            timestamp_from_us: Some(1_000_000),
            timestamp_to_us:   Some(2_000_000),
            priority:   None,
            pagination: None,
        };
        let query = JournalQuery::from(req);
        assert_eq!(query.timestamp_from, Some(1_000_000));
        assert_eq!(query.timestamp_to,   Some(2_000_000));
    }

    #[test]
    fn absent_timestamps_are_none() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: None,
        };
        let query = JournalQuery::from(req);
        assert!(query.timestamp_from.is_none());
        assert!(query.timestamp_to.is_none());
    }

    #[test]
    fn absent_priority_defaults_to_info() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: None,
        };
        assert_eq!(JournalQuery::from(req).priority, JournalPriority::Info);
    }

    #[test]
    fn absent_pagination_uses_default() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: None,
        };
        assert_eq!(JournalQuery::from(req).pagination, PaginationRequest::default());
    }

    #[test]
    fn pagination_page_size_is_passed_through() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: Some(PageRequest { page_size: 25, page_token: None, page_direction: 0 }),
        };
        assert_eq!(JournalQuery::from(req).pagination.page_size, 25);
    }

    #[test]
    fn zero_page_size_uses_default() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: Some(PageRequest { page_size: 0, page_token: None, page_direction: 0 }),
        };
        assert_eq!(JournalQuery::from(req).pagination.page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn unit_name_is_empty() {
        let req = GetJournalsRequest {
            timestamp_from_us: None,
            timestamp_to_us:   None,
            priority:   None,
            pagination: None,
        };
        assert_eq!(JournalQuery::from(req).unit_name, "");
    }
}
