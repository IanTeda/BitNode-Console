use crate::domains;

/// A page of journal entries returned by [`fetch_entries`].
#[derive(Debug)]
pub struct JournalPage {
    /// Entries in ascending timestamp order.
    pub entries: Vec<domains::JournalEntry>,

    /// Pagination metadata for this page.
    pub pagination: lib_core::domains::pagination::PaginationResponse,
}
