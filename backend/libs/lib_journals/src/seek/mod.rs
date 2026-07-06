//! Journal seek entry point — applies the unit filter and dispatches by pagination direction.

use crate::{JournalConnection, JournalPage, JournalQuery, Result};
use lib_core::domains::pagination::Direction;

mod backward;
mod forward;

impl<'a> JournalQuery<'a> {
    pub fn seek(&self, conn: &mut JournalConnection) -> Result<JournalPage> {
        // Apply unit filter before reading any entries.
        if !self.unit_name.is_empty() {
            conn.match_unit(self.unit_name)?;
        }

        // Seek forward or backward based on the pagination direction.
        match self.pagination.direction {
            Direction::Forward => self.seek_forward(conn),
            Direction::Backward => self.seek_backward(conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_core::domains::pagination::{Direction, PaginationRequest};

    use crate::{JournalConnection, JournalPriority, JournalQuery};

    /// Inject a line into the system journal under `identifier` at the given syslog priority.
    fn inject(identifier: &str, priority: &str, message: &str) {
        use std::io::Write as _;
        let mut child = std::process::Command::new("systemd-cat")
            .args(["-t", identifier, "-p", priority])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("systemd-cat must be available");
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(message.as_bytes()).ok();
        }
        child.wait().expect("systemd-cat must complete");
    }

    fn open() -> JournalConnection {
        JournalConnection::open().expect("system journal must open")
    }

    #[test]
    fn forward_direction_returns_entries() {
        let id = "lib-jd-mod-fwd";
        inject(id, "info", "fwd-msg");

        let mut conn = open();
        let query = JournalQuery::new(
            "lib-jd-mod-fwd.service",
            JournalPriority::Debug,
            PaginationRequest {
                page_size: 50,
                page_token: None,
                direction: Direction::Forward,
            },
        );
        let page = query.seek(&mut conn).expect("seek must succeed");
        assert!(
            !page.entries.is_empty(),
            "expected entries with Forward direction"
        );
    }

    #[test]
    fn backward_direction_returns_entries() {
        let id = "lib-jd-mod-bwd";
        inject(id, "info", "bwd-msg");

        let mut conn = open();
        let query = JournalQuery::new(
            "lib-jd-mod-bwd.service",
            JournalPriority::Debug,
            PaginationRequest {
                page_size: 50,
                page_token: None,
                direction: Direction::Backward,
            },
        );
        let page = query.seek(&mut conn).expect("seek must succeed");
        assert!(
            !page.entries.is_empty(),
            "expected entries with Backward direction"
        );
    }

    #[test]
    fn unit_filter_isolates_entries_by_identifier() {
        let id_a = "lib-jd-mod-filter-a";
        let id_b = "lib-jd-mod-filter-b";
        inject(id_a, "info", "msg-from-a");
        inject(id_b, "info", "msg-from-b");

        let mut conn = open();
        let query = JournalQuery::new(
            "lib-jd-mod-filter-a.service",
            JournalPriority::Debug,
            PaginationRequest {
                page_size: 50,
                page_token: None,
                direction: Direction::Forward,
            },
        );
        let page = query.seek(&mut conn).expect("seek must succeed");

        for entry in &page.entries {
            assert_eq!(
                entry.unit, id_a,
                "expected only entries from '{id_a}', got unit '{}'",
                entry.unit,
            );
        }
    }

    #[test]
    fn empty_unit_name_skips_filter_and_returns_entries() {
        let mut conn = open();
        let query = JournalQuery::new(
            "",
            JournalPriority::Debug,
            PaginationRequest {
                page_size: 10,
                page_token: None,
                direction: Direction::Forward,
            },
        );
        let page = query.seek(&mut conn).expect("seek must succeed");
        assert!(
            !page.entries.is_empty(),
            "expected entries when no unit filter is applied"
        );
    }
}
