//! Query parameters for fetching journal entries.
//!
//! [`Query`] bundles all parameters that control a single journal seek: which
//! systemd unit to filter on, optional timestamp bounds, a minimum priority
//! threshold, and cursor-based pagination settings. Build one with
//! [`Query::new`] for full control, or [`Query::with_unit`] for a quick
//! defaults-based query against a single unit.

use lib_core::domains::pagination::PaginationRequest;

/// Systemd unit name used when no specific unit is requested.
const DEFAULT_UNIT_NAME: &str = "";

/// Priority threshold applied when no specific priority is requested.
const DEFAULT_PRIORITY: crate::Priority = crate::Priority::Info;

/// Parameters controlling which journal entries to return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query<'a> {
    /// Systemd unit name to filter on, e.g. `"bitcoind.service"`.
    ///
    /// Matched against `_SYSTEMD_UNIT`. The `.service` suffix is also stripped
    /// and matched against `SYSLOG_IDENTIFIER`, covering the `systemd-cat -t`
    /// development workflow described in [`BitcoinDaemonSettings`].
    ///
    /// [`BitcoinDaemonSettings`]: lib_settings::BitcoinDaemonSettings
    pub unit_name: &'a str,

    /// Lower timestamp bound in microseconds since the Unix epoch (inclusive).
    ///
    /// When `None` the seek starts at the head (forward) or tail (backward) of
    /// the journal, or resumes from `pagination.page_token` if one is set.
    pub timestamp_from: Option<i64>,

    /// Upper timestamp bound in microseconds since the Unix epoch (inclusive).
    ///
    /// Entries with a timestamp strictly greater than this value are excluded.
    /// `None` means no upper bound.
    pub timestamp_to: Option<i64>,

    /// Maximum priority level to include; entries with a lower severity are dropped.
    ///
    /// Priority ordering follows syslog conventions: lower numeric value means
    /// higher severity (e.g. `Emergency = 0`, `Debug = 7`). An entry is
    /// included when `entry.priority <= self.priority`.
    pub priority: crate::Priority,

    /// Page size, resume cursor, and read direction for this request.
    pub pagination: PaginationRequest,
}

impl<'a> Default for Query<'a> {
    /// Returns a `Query` with an empty unit filter, no timestamp bounds,
    /// [`Priority::Info`](crate::Priority::Info) threshold, and default
    /// pagination settings.
    fn default() -> Self {
        Self {
            unit_name: DEFAULT_UNIT_NAME,
            timestamp_from: None,
            timestamp_to: None,
            priority: DEFAULT_PRIORITY,
            pagination: PaginationRequest::default(),
        }
    }
}

impl<'a> Query<'a> {
    /// Create a new `Query` with explicit parameters.
    ///
    /// Timestamp bounds default to `None` (unbounded). Use struct-update
    /// syntax (`Query { timestamp_from: Some(ts), ..Query::new(...) }`) to set
    /// them after construction.
    ///
    /// # Arguments
    ///
    /// * `unit_name` — Systemd unit name to filter on.
    /// * `priority` — Maximum priority level to include.
    /// * `pagination` — Page size, resume cursor, and read direction.
    #[must_use]
    pub fn new(
        unit_name: &'a str,
        priority: crate::Priority,
        pagination: PaginationRequest,
    ) -> Self {
        Self {
            unit_name,
            timestamp_from: None,
            timestamp_to: None,
            priority,
            pagination,
        }
    }

    /// Create a `Query` for `unit_name` using defaults for everything else.
    ///
    /// Equivalent to `Query { unit_name, ..Query::default() }`.
    #[must_use]
    pub fn with_unit(unit_name: &'a str) -> Self {
        Self {
            unit_name,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_core::domains::pagination::{DEFAULT_PAGE_SIZE, Direction, PaginationRequest};

    use crate::{Priority, Query};

    fn pagination(size: u32, token: Option<&str>, direction: Direction) -> PaginationRequest {
        PaginationRequest {
            page_size: size,
            page_token: token.map(str::to_owned),
            direction,
        }
    }

    // --- default ---

    #[test]
    fn default_unit_name_is_empty() {
        assert_eq!(Query::default().unit_name, "");
    }

    #[test]
    fn default_priority_is_info() {
        assert_eq!(Query::default().priority, Priority::Info);
    }

    #[test]
    fn default_timestamps_are_none() {
        let q = Query::default();
        assert!(q.timestamp_from.is_none());
        assert!(q.timestamp_to.is_none());
    }

    #[test]
    fn default_pagination_uses_default_page_size() {
        assert_eq!(Query::default().pagination.page_size, DEFAULT_PAGE_SIZE);
    }

    // --- new ---

    #[test]
    fn new_sets_unit_name_priority_and_pagination() {
        let pag = pagination(10, None, Direction::Forward);
        let q = Query::new("bitcoind.service", Priority::Error, pag.clone());

        assert_eq!(q.unit_name, "bitcoind.service");
        assert_eq!(q.priority, Priority::Error);
        assert_eq!(q.pagination, pag);
    }

    #[test]
    fn new_timestamps_default_to_none() {
        let q = Query::new("svc", Priority::Info, PaginationRequest::default());
        assert!(q.timestamp_from.is_none());
        assert!(q.timestamp_to.is_none());
    }

    #[test]
    fn new_supports_struct_update_to_set_timestamps() {
        let q = Query {
            timestamp_from: Some(1_000_000),
            timestamp_to: Some(2_000_000),
            ..Query::new(
                "bitcoind.service",
                Priority::Debug,
                PaginationRequest::default(),
            )
        };

        assert_eq!(q.timestamp_from, Some(1_000_000));
        assert_eq!(q.timestamp_to, Some(2_000_000));
        assert_eq!(q.unit_name, "bitcoind.service");
    }

    // --- with_unit ---

    #[test]
    fn with_unit_sets_unit_name() {
        assert_eq!(
            Query::with_unit("bitcoind.service").unit_name,
            "bitcoind.service"
        );
    }

    #[test]
    fn with_unit_keeps_default_priority() {
        assert_eq!(Query::with_unit("svc").priority, Priority::Info);
    }

    #[test]
    fn with_unit_keeps_default_pagination() {
        let q = Query::with_unit("svc");
        assert_eq!(q.pagination, PaginationRequest::default());
    }

    // --- equality ---

    #[test]
    fn equal_queries_compare_equal() {
        let pag = pagination(5, Some("tok"), Direction::Backward);
        let a = Query::new("svc", Priority::Warning, pag.clone());
        let b = Query::new("svc", Priority::Warning, pag);
        assert_eq!(a, b);
    }

    #[test]
    fn queries_differing_in_unit_name_are_not_equal() {
        let a = Query::with_unit("a.service");
        let b = Query::with_unit("b.service");
        assert_ne!(a, b);
    }
}
