//! Journal seek dispatcher.
//!
//! Provides [`Query::seek`], the single entry point for reading a page of
//! journal entries. It applies the unit match filter on the [`Connection`],
//! then delegates to [`Query::seek_forward`] or [`Query::seek_backward`]
//! based on the requested [`Direction`].

use crate::{Connection, Page, Query, Result};
use lib_core::domains::pagination::Direction;

impl<'a> Query<'a> {
    /// Read a page of journal entries according to `self`.
    ///
    /// Applies the unit match filter when `unit_name` is non-empty, then
    /// dispatches to [`seek_forward`](Query::seek_forward) or
    /// [`seek_backward`](Query::seek_backward) based on
    /// `pagination.direction`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if applying the unit filter or any
    /// underlying journal operation fails.
    pub fn seek(&self, conn: &mut Connection) -> Result<Page> {
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

    use crate::{Connection, Priority, Query};

    /// Inject `message` into the system journal under `identifier` at `priority`.
    ///
    /// Uses `systemd-cat` via stdin so the entry is written as journal data,
    /// not executed as a shell command.
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

    /// Open the system journal, panicking if it cannot be opened.
    fn open() -> Connection {
        Connection::open().expect("system journal must open")
    }

    /// Build a `PaginationRequest` with the given direction and no cursor.
    fn pag(size: u32, direction: Direction) -> PaginationRequest {
        PaginationRequest {
            page_size: size,
            page_token: None,
            direction,
        }
    }

    #[test]
    fn forward_direction_returns_entries() {
        let id = "lib-jd-mod-fwd";
        inject(id, "info", "fwd-msg");

        let mut conn = open();
        let page = Query::new("lib-jd-mod-fwd.service", Priority::Debug, pag(50, Direction::Forward))
            .seek(&mut conn)
            .expect("seek must succeed");

        assert!(!page.entries.is_empty(), "expected entries with Forward direction");
    }

    #[test]
    fn backward_direction_returns_entries() {
        let id = "lib-jd-mod-bwd";
        inject(id, "info", "bwd-msg");

        let mut conn = open();
        let page = Query::new("lib-jd-mod-bwd.service", Priority::Debug, pag(50, Direction::Backward))
            .seek(&mut conn)
            .expect("seek must succeed");

        assert!(!page.entries.is_empty(), "expected entries with Backward direction");
    }

    #[test]
    fn unit_filter_isolates_entries_by_identifier() {
        let id_a = "lib-jd-mod-filter-a";
        let id_b = "lib-jd-mod-filter-b";
        inject(id_a, "info", "msg-from-a");
        inject(id_b, "info", "msg-from-b");

        let mut conn = open();
        let page = Query::new("lib-jd-mod-filter-a.service", Priority::Debug, pag(50, Direction::Forward))
            .seek(&mut conn)
            .expect("seek must succeed");

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
        let page = Query::new("", Priority::Debug, pag(10, Direction::Forward))
            .seek(&mut conn)
            .expect("seek must succeed");

        assert!(!page.entries.is_empty(), "expected entries when no unit filter is applied");
    }

    #[test]
    fn unknown_unit_returns_empty_page() {
        let mut conn = open();
        let page = Query::new(
            "lib-jd-seek-no-such-unit-xyzzy.service",
            Priority::Debug,
            pag(50, Direction::Forward),
        )
        .seek(&mut conn)
        .expect("seek must succeed");

        assert!(page.entries.is_empty(), "unknown unit must return no entries");
    }
}
