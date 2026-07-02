use systemd::journal::JournalSeek;

use crate::{JournalConnection, JournalEntry, JournalPage, Query, Result};

impl<'a> Query<'a> {
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
        //--- Seek to the cursor position or head, then read forward to collect entries.
        match self.after_cursor {
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

        // Convert the limit to usize
        let limit = usize::try_from(self.page_size)?;

        // Create a vector to hold the entries
        let mut entries = Vec::new();

        // Read entries until the limit is reached or the end of the journal is reached.
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

        // Get the next cursor position, if any (for use in pagination).
        let next_cursor = entries.last().and_then(|e| e.cursor.clone());

        Ok(crate::JournalPage {
            entries,
            next_cursor,
        })
    }
}
