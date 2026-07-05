//! Query parameters for fetching journal entries.

use crate::domains::JournalPriority;
use lib_core::domains::pagination::PaginationRequest;

/// Default unit name to use when no specific unit name is provided.
const DEFAULT_UNIT_NAME: &str = "";

/// Default priority level to use when no specific priority is provided.
const DEFAULT_PRIORITY: JournalPriority = JournalPriority::Info;

/// Parameters controlling which journal entries to return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalQuery<'a> {
    /// Systemd unit name to filter on, e.g. `"bitcoind.service"`.
    ///
    /// Matched against `_SYSTEMD_UNIT`. The `.service` suffix is also stripped
    /// and matched against `SYSLOG_IDENTIFIER`, covering the `systemd-cat -t`
    /// development workflow described in [`BitcoinDaemonSettings`].
    ///
    /// [`BitcoinDaemonSettings`]: lib_settings::BitcoinDaemonSettings
    pub unit_name: &'a str,

    /// Timestamp to start from, in microseconds since the Unix epoch.
    pub timestamp_from: Option<i64>,

    /// Timestamp to end at, in microseconds since the Unix epoch.
    pub timestamp_to: Option<i64>,

    /// Minimum priority level to return.
    pub priority: JournalPriority,

    /// Page size, cursor, and direction for this request.
    pub pagination: PaginationRequest,
}

impl<'a> Default for JournalQuery<'a> {
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

impl<'a> JournalQuery<'a> {
    /// Create a new `Query` with explicit parameters.
    ///
    /// # Arguments
    ///
    /// * `unit_name` — Systemd unit name to filter on.
    /// * `priority` — Minimum severity level to return.
    /// * `pagination` — Page size, cursor, and direction for this request.
    #[must_use]
    pub fn new(
        unit_name: &'a str,
        priority: JournalPriority,
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

    /// Create a `Query` for the given unit name, using defaults for everything else.
    #[must_use]
    pub fn with_unit(unit_name: &'a str) -> Self {
        Self {
            unit_name,
            ..Default::default()
        }
    }
}
