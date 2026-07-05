use lib_core::domains::pagination::PaginationResponse;
use systemd::journal::JournalSeek;

use crate::{JournalConnection, JournalEntry, JournalPage, JournalQuery, Result};

impl<'a> JournalQuery<'a> {
    /// Seek backward through the journal, reading entries from newest to oldest.
    ///
    /// Starts at [`JournalSeek::Tail`] when no cursor is provided, or just before `page_token`
    /// when resuming a previous page. Entries are reversed before returning so they are always
    /// in ascending timestamp order.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if any underlying journal operation fails.
    pub fn seek_backward(&self, conn: &mut JournalConnection) -> Result<JournalPage> {
        let limit = usize::try_from(self.pagination.page_size)?;
        let mut entries = Vec::with_capacity(limit);

        // Check if there is an existing cursor to resume from, and seek to that position if so.
        match self.pagination.page_token.as_deref() {
            // When resuming from a cursor, seek to that position and step before the cursor entry.
            Some(cursor) => {
                conn.journal.seek(JournalSeek::Cursor {
                    cursor: cursor.to_owned(),
                })?;
                // Step before the cursor entry so it isn't repeated on this page.
                conn.journal.previous()?;
            },
            // When no cursor is provided, start from the tail and seek backwards.
            None => conn.journal.seek(JournalSeek::Tail)?,
        }

        // --- Check if there is a previous entry before returning the prev_page_token. We are going
        // backwards, so the previous entry is the one after the current position.
        let prev_page_token = if conn.journal.next()? != 0 {
            conn.journal.cursor().ok()
        } else {
            None
        };

        // --- Read entries backwards from the current position and add them to the entries list.
        while entries.len() < limit {
            let Some(record) = conn.journal.previous_entry()? else {
                break;
            };
            let timestamp_us = i64::try_from(conn.journal.timestamp_usec()?)?;
            let mut entry = JournalEntry::from_record(&record, timestamp_us);
            // __CURSOR is metadata, not a data field — fetch it separately after positioning.
            entry.cursor = conn.journal.cursor().ok();
            if entry.priority <= self.priority {
                entries.push(entry);
            }
        }

        // Restore chronological (oldest-first) order after reading backward.
        entries.reverse();

        // --- Check if there is a next entry before returning the prev_page_token. We are going
        // backwards, so the next entry is the one before the current position.
        let next_page_token = if conn.journal.previous()? != 0 {
            conn.journal.cursor().ok()
        } else {
            None
        };

        Ok(JournalPage {
            entries,
            pagination: PaginationResponse {
                next_page_token,
                prev_page_token,
            },
        })
    }
}
