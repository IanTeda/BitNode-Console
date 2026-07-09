//! Query parameters for fetching journal entries.

use lib_core::domains::pagination::PaginationRequest;

/// Default unit name to use when no specific unit name is provided.
const DEFAULT_UNIT_NAME: &str = "";

/// Default priority level to use when no specific priority is provided.
const DEFAULT_PRIORITY: crate::Priority = crate::Priority::Info;

/// Default number of tail lines to use when no specific number is provided.
const DEFAULT_TAIL_LINES: u32 = 30;

/// Parameters controlling which journal entries to follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowTail<'a> {
    /// Systemd unit name to filter on, e.g. `"bitcoind.service"`.
    pub unit_name: &'a str,

    /// Minimum priority level to return.
    pub priority: crate::Priority,

    /// Number of existing tail entries to replay before streaming new entries.
    pub tail_lines: u32,
}

impl Default for FollowTail<'_> {
    fn default() -> Self {
        Self {
            unit_name: DEFAULT_UNIT_NAME,
            priority: DEFAULT_PRIORITY,
            tail_lines: DEFAULT_TAIL_LINES,
        }
    }
}

impl<'a> FollowTail<'a> {
    /// Create a new [`JournalFollowTail`] with explicit parameters.
    ///
    /// # Arguments
    ///
    /// * `unit_name` — Systemd unit name to filter on.
    /// * `priority` — Minimum severity level to return.
    /// * `tail_lines` — Number of existing tail entries to replay first.
    #[must_use]
    pub const fn new(unit_name: &'a str, priority: crate::Priority, tail_lines: u32) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_constants() {
        let follow = FollowTail::default();
        assert_eq!(follow.unit_name, DEFAULT_UNIT_NAME);
        assert_eq!(follow.priority, DEFAULT_PRIORITY);
        assert_eq!(follow.tail_lines, DEFAULT_TAIL_LINES);
    }

    #[test]
    fn new_stores_all_fields() {
        let follow = FollowTail::new("bitcoind.service", crate::Priority::Warning, 50);
        assert_eq!(follow.unit_name, "bitcoind.service");
        assert_eq!(follow.priority, crate::Priority::Warning);
        assert_eq!(follow.tail_lines, 50);
    }

    #[test]
    fn with_unit_sets_unit_name_and_keeps_defaults() {
        let follow = FollowTail::with_unit("bitcoind.service");
        assert_eq!(follow.unit_name, "bitcoind.service");
        assert_eq!(follow.priority, DEFAULT_PRIORITY);
        assert_eq!(follow.tail_lines, DEFAULT_TAIL_LINES);
    }
}
