//! Query parameters for fetching journal entries.

use crate::domains::JournalPriority;
use lib_core::domains::pagination::PaginationRequest;

/// Default unit name to use when no specific unit name is provided.
const DEFAULT_UNIT_NAME: &str = "";

/// Default priority level to use when no specific priority is provided.
const DEFAULT_PRIORITY: JournalPriority = JournalPriority::Info;

/// Default number of tail lines to use when no specific number is provided.
const DEFAULT_TAIL_LINES: u32 = 30;

/// Parameters controlling which journal entries to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFollowTail<'a> {
    /// Systemd unit name to filter on, e.g. `"bitcoind.service"`.
    pub unit_name: &'a str,
    /// Minimum priority level to return.
    pub priority: JournalPriority,
    /// Number of existing tail entries to replay before streaming new entries.
    pub tail_lines: u32,
}

impl<'a> Default for JournalFollowTail<'a> {
    fn default() -> Self {
        Self {
            unit_name: DEFAULT_UNIT_NAME,
            priority: DEFAULT_PRIORITY,
            tail_lines: DEFAULT_TAIL_LINES,
        }
    }
}

impl<'a> JournalFollowTail<'a> {
    /// Create a new [`JournalFollowTail`] with explicit parameters.
    ///
    /// # Arguments
    ///
    /// * `unit_name` — Systemd unit name to filter on.
    /// * `priority` — Minimum severity level to return.
    /// * `tail_lines` — Number of existing tail entries to replay first.
    #[must_use]
    pub fn new(unit_name: &'a str, priority: JournalPriority, tail_lines: u32) -> Self {
        Self {
            unit_name,
            priority,
            tail_lines,
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
