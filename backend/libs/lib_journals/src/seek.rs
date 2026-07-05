use lib_core::domains::pagination::PaginationResponse;
use systemd::journal::JournalSeek;

use crate::{JournalConnection, JournalEntry, JournalPage, JournalQuery, Result};

impl<'a> JournalQuery<'a> {
    /// Fetch journal entries and build a [`crate::JournalPage`] from them.
    ///
    /// Seeks to the head for the first page, or just past `after_cursor` for
    /// subsequent pages, then reads forward until `limit` priority-passing
    /// entries have been collected or the journal is exhausted.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if any underlying journal operation fails.
    pub fn seek(&self, conn: &mut JournalConnection) -> Result<JournalPage> {
        //--- Apply unit filter before reading any entries.
        if !self.unit_name.is_empty() {
            conn.match_unit(self.unit_name)?;
        }

        //--- Seek to the cursor position or head, then read forward to collect entries.
        match self.pagination.page_token.as_deref() {
            Some(cursor) => {
                conn.journal.seek(JournalSeek::Cursor {
                    cursor: cursor.to_owned(),
                })?;
                // Move onto the cursor entry; the loop's first next_entry()
                // will then advance past it to begin the new page.
                conn.journal.next()?;
            },
            None => conn.journal.seek(JournalSeek::Head)?,
        }

        let limit = usize::try_from(self.pagination.page_size)?;
        let mut entries = Vec::new();

        while entries.len() < limit {
            let Some(record) = conn.journal.next_entry()? else {
                break;
            };
            let timestamp_us = i64::try_from(conn.journal.timestamp_usec()?)?;
            let entry = JournalEntry::from_record(&record, timestamp_us);
            if entry.priority <= self.priority {
                entries.push(entry);
            }
        }

        let next_page_token = entries.last().and_then(|e| e.cursor.clone());

        Ok(JournalPage {
            entries,
            pagination: PaginationResponse {
                next_page_token,
                prev_page_token: None,
            },
        })
    }
}
