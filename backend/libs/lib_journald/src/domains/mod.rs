//! Journal domain types.

mod entry;
mod page;
mod priority;

pub use entry::JournalEntry;
pub use page::JournalPage;
pub use priority::JournalPriority;
