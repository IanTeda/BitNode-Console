//! Forward (oldest-first) journal seek implementation.

use lib_core::domains::pagination::PaginationResponse;
use systemd::journal::JournalSeek;

use crate::{Entry, Connection, Page, Query, Result};

impl<'a> Query<'a> {
    /// Seek forward through the journal, reading entries from oldest to newest.
    ///
    /// Starts at [`JournalSeek::Head`] when no cursor is provided, or just past `page_token`
    /// when resuming a previous page.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if any underlying journal operation fails.
    pub fn seek_forward(&self, conn: &mut Connection) -> Result<Page> {
        let limit = usize::try_from(self.pagination.page_size)?;
        let mut entries = Vec::with_capacity(limit);

        // --- 01. Seek to Cursor Position

        // Check if there is an existing cursor to resume from, and seek to that position if so.
        match self.pagination.page_token.as_deref() {
            // When resuming from a cursor, seek to that position and step next to the cursor entry.
            Some(cursor) => {
                conn.journal.seek(JournalSeek::Cursor {
                    cursor: cursor.to_owned(),
                })?;
                // Step next to the cursor entry so it isn't repeated on this page.
                conn.journal.next()?;
            },
            // When no cursor is provided, start from timestamp_from or use the head and seek forward.
            None => match self.timestamp_from.and_then(|ts| u64::try_from(ts).ok()) {
                Some(usec) => conn.journal.seek(JournalSeek::ClockRealtime { usec })?,
                None => conn.journal.seek(JournalSeek::Head)?,
            },
        }

        // --- 02. Set Previous Page Token

        // Check if there is a previous entry before returning the prev_page_token.
        let prev_page_token = if conn.journal.previous()? != 0 {
            conn.journal.cursor().ok()
        } else {
            None
        };

        // --- 03. Build Entries List

        // Read entries forward from the current position and add them to the entries list.
        while entries.len() < limit {
            let Some(record) = conn.journal.next_entry()? else {
                break;
            };
            let timestamp_us = i64::try_from(conn.journal.timestamp_usec()?)?;
            // Stop once we've advanced past the upper timestamp bound.
            if let Some(to) = self.timestamp_to {
                if timestamp_us > to {
                    break;
                }
            }
            let mut entry = Entry::from_record(&record, timestamp_us);
            // __CURSOR is metadata, not a data field — fetch it separately after positioning.
            entry.cursor = conn.journal.cursor().ok();
            if entry.priority <= self.priority {
                entries.push(entry);
            }
        }

        // --- 04. Set Next Page Token

        // Check if there is a next entry before returning the next_page_token.
        let next_page_token = if conn.journal.next()? != 0 {
            conn.journal.cursor().ok()
        } else {
            None
        };

        Ok(Page {
            entries,
            pagination: PaginationResponse {
                next_page_token,
                prev_page_token,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use lib_core::domains::pagination::{Direction, PaginationRequest};

    use crate::{Connection, Priority, Query};

    /// Inject a line into the system journal under `identifier` at the given syslog priority.
    ///
    /// Uses stdin piping so the message is written as journal data, not executed as a command.
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

    fn make_query(unit: &'static str, page_size: u32, token: Option<String>) -> Query<'static> {
        Query::new(
            unit,
            Priority::Debug,
            PaginationRequest {
                page_size,
                page_token: token,
                direction: Direction::Forward,
            },
        )
    }

    /// Run a forward seek through the proper entry point so the unit filter is applied.
    fn forward_seek(query: Query<'_>, conn: &mut Connection) -> crate::Result<crate::Page> {
        query.seek(conn)
    }

    fn open() -> Connection {
        Connection::open().expect("system journal must open")
    }

    #[test]
    fn unknown_unit_returns_empty_page() {
        let mut conn = open();
        let query = make_query("lib-jd-fwd-no-such-unit.service", 10, None);
        let page = forward_seek(query, &mut conn).expect("seek must succeed");
        assert!(page.entries.is_empty());
        assert!(page.pagination.next_page_token.is_none());
    }

    #[test]
    fn page_size_zero_returns_empty_page() {
        let mut conn = open();
        let query = make_query("lib-jd-fwd-limit-zero.service", 0, None);
        let page = forward_seek(query, &mut conn).expect("seek must succeed");
        assert!(page.entries.is_empty());
    }

    #[test]
    fn entries_are_in_ascending_timestamp_order() {
        // Inject using the bare identifier; query with .service so SYSLOG_IDENTIFIER is matched.
        let id = "lib-jd-fwd-order";
        for i in 0..3 {
            inject(id, "info", &format!("order-msg-{i}"));
        }

        let mut conn = open();
        let page = forward_seek(make_query("lib-jd-fwd-order.service", 50, None), &mut conn)
            .expect("seek must succeed");

        assert!(!page.entries.is_empty(), "expected entries after injection");
        let timestamps: Vec<i64> = page.entries.iter().map(|e| e.timestamp_us).collect();
        let mut sorted = timestamps.clone();
        sorted.sort_unstable();
        assert_eq!(
            timestamps, sorted,
            "entries must be in ascending timestamp order"
        );
    }

    #[test]
    fn page_size_is_respected_and_next_token_is_set() {
        let id = "lib-jd-fwd-limit";
        for i in 0..5 {
            inject(id, "info", &format!("limit-msg-{i}"));
        }

        let mut conn = open();
        let page = forward_seek(make_query("lib-jd-fwd-limit.service", 3, None), &mut conn)
            .expect("seek must succeed");

        assert!(
            page.entries.len() <= 3,
            "returned {} entries, expected at most 3",
            page.entries.len(),
        );
        // There are more entries beyond the page, so a next token must be present.
        assert!(
            page.pagination.next_page_token.is_some(),
            "expected a next_page_token when entries were truncated",
        );
    }

    #[test]
    fn first_page_from_head_has_no_prev_page_token() {
        let id = "lib-jd-fwd-no-prev";
        inject(id, "info", "head-msg");

        let mut conn = open();
        let page = forward_seek(
            make_query("lib-jd-fwd-no-prev.service", 50, None),
            &mut conn,
        )
        .expect("seek must succeed");

        assert!(
            page.pagination.prev_page_token.is_none(),
            "page starting at head must have no prev_page_token",
        );
    }

    #[test]
    fn cursor_pagination_second_page_differs_from_first() {
        let id = "lib-jd-fwd-cursor";
        for i in 0..6 {
            inject(id, "info", &format!("cursor-msg-{i}"));
        }

        let mut conn = open();
        let first = forward_seek(make_query("lib-jd-fwd-cursor.service", 3, None), &mut conn)
            .expect("first seek must succeed");
        assert!(!first.entries.is_empty(), "expected entries on first page");

        let token = first
            .pagination
            .next_page_token
            .expect("expected a next_page_token after first page");

        let mut conn2 = open();
        let second = forward_seek(
            make_query("lib-jd-fwd-cursor.service", 3, Some(token)),
            &mut conn2,
        )
        .expect("second seek must succeed");

        let first_cursors: Vec<_> = first.entries.iter().map(|e| e.cursor.as_deref()).collect();
        let second_cursors: Vec<_> = second.entries.iter().map(|e| e.cursor.as_deref()).collect();
        assert!(
            first_cursors.iter().all(|c| !second_cursors.contains(c)),
            "second page must not repeat entries from the first page",
        );
    }

    #[test]
    fn priority_threshold_filters_out_less_severe_entries() {
        let id = "lib-jd-fwd-priority";
        inject(id, "info", "info-msg-should-be-excluded");

        let mut conn = open();
        let query = Query::new(
            "lib-jd-fwd-priority.service",
            Priority::Error,
            PaginationRequest {
                page_size: 50,
                page_token: None,
                direction: Direction::Forward,
            },
        );
        let page = forward_seek(query, &mut conn).expect("seek must succeed");

        for entry in &page.entries {
            assert!(
                entry.priority <= Priority::Error,
                "entry with priority {:?} must not exceed Error threshold",
                entry.priority,
            );
        }
    }
}
