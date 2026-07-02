use crate::domains;

/// Default unit name to use when no specific unit name is provided.
const DEFAULT_UNIT_NAME: &str = "";

/// Default limit to use when no specific limit is provided.
const DEFAULT_LIMIT: u64 = 100;

/// Default priority level to use when no specific priority is provided.
const DEFAULT_PRIORITY: domains::JournalPriority = domains::JournalPriority::Info;

/// Default cursor to use when no specific cursor is provided.
const DEFAULT_AFTER_CURSOR: Option<&str> = None;

/// Settings controlling which journal entries to return from the query.
pub struct Query<'a> {
    /// Systemd unit name to filter on, e.g. `"bitcoind.service"`.
    ///
    /// Matched against `_SYSTEMD_UNIT`. The `.service` suffix is also stripped
    /// and matched against `SYSLOG_IDENTIFIER`, covering the `systemd-cat -t`
    /// development workflow described in [`BitcoinDaemonSettings`].
    ///
    /// [`BitcoinDaemonSettings`]: lib_settings::BitcoinDaemonSettings
    pub unit_name: &'a str,

    /// Minimum priority level to return.
    pub priority: domains::JournalPriority,

    /// Maximum number of entries to return in one page.
    pub page_size: u64,

    /// Opaque cursor returned by a prior call; the page starts *after* this entry.
    ///
    /// Pass `None` to start from the oldest available entry.
    pub after_cursor: Option<&'a str>,
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::new(
            DEFAULT_UNIT_NAME,
            DEFAULT_PRIORITY,
            DEFAULT_LIMIT,
            DEFAULT_AFTER_CURSOR,
        )
    }
}

impl<'a> Query<'a> {
    /// Creates a new `Query` with the given unit name and default priority, limit, and cursor.
    ///
    /// # Arguments
    ///
    /// * `unit_name` - The systemd unit name to filter on.
    /// * `priority` - The minimum priority level to return.
    /// * `limit` - The maximum number of entries to return in one page.
    /// * `after_cursor` - The opaque cursor returned by a prior call; the page starts *after* this entry.
    ///
    /// Pass `None` to start from the oldest available entry.
    #[must_use]
    pub fn new(
        unit_name: &'a str,
        priority: domains::JournalPriority,
        page_size: u64,
        after_cursor: Option<&'a str>,
    ) -> Self {
        Self {
            unit_name,
            priority,
            page_size,
            after_cursor,
        }
    }

    /// Creates a new `Query` with the given unit name and default priority, limit, and cursor.
    ///
    /// # Arguments
    ///
    /// * `unit_name` - The systemd unit name to filter on.
    #[must_use]
    pub fn new_with_unit_name(unit_name: &'a str) -> Self {
        Self {
            unit_name,
            ..Default::default()
        }
    }
}
