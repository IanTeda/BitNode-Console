use lib_core::domains::pagination::PaginationResponse;
use systemd::journal::JournalSeek;

use crate::{JournalConnection, JournalEntry, JournalPage, JournalQuery, Result};

impl<'a> JournalQuery<'a> {
    /// Seek forward through the journal, reading entries from oldest to newest.
    ///
    /// Starts at [`JournalSeek::Head`] when no cursor is provided, or just past `page_token`
    /// when resuming a previous page.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if any underlying journal operation fails.
    pub fn seek_forward(&self, conn: &mut JournalConnection) -> Result<JournalPage> {
        let limit = usize::try_from(self.pagination.page_size)?;
        let mut entries = Vec::with_capacity(limit);

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
            // When no cursor is provided, start from the head and seek forward.
            None => conn.journal.seek(JournalSeek::Head)?,
        }

        // --- Check if there is a previous entry before returning the prev_page_token.
        let prev_page_token = if conn.journal.previous()? != 0 {
            conn.journal.cursor().ok()
        } else {
            None
        };

        // --- Read entries forward from the current position and add them to the entries list.
        while entries.len() < limit {
            let Some(record) = conn.journal.next_entry()? else {
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

        // --- Check if there is a next entry before returning the next_page_token.
        let next_page_token = if conn.journal.next()? != 0 {
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
