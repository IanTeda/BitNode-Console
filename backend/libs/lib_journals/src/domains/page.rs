use crate::domains;

/// A page of journal entries returned by [`fetch_entries`].
pub struct JournalPage {
    /// Entries in ascending timestamp order.
    pub entries: Vec<domains::JournalEntry>,

    /// Cursor of the last entry in this page.
    ///
    /// Pass as [`FetchOptions::after_cursor`] to retrieve the next page.
    /// `None` when the page is empty (log exhausted or no matching entries).
    pub next_cursor: Option<String>,
}
