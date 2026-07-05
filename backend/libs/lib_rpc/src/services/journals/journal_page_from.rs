//! Conversion from [`lib_journals::JournalPage`] into [`GetJournalsResponse`].

use lib_journals::{JournalEntry, JournalPage, JournalPriority};

use crate::generated_protos::journals::{GetJournalsResponse, JournalsEntry, Priority};
use crate::services::pagination::PageResponse;

// ── JournalPriority → proto Priority ─────────────────────────────────────────

impl From<JournalPriority> for Priority {
    fn from(p: JournalPriority) -> Self {
        match p {
            JournalPriority::Emergency => Self::Emergency,
            JournalPriority::Alert     => Self::Alert,
            JournalPriority::Critical  => Self::Critical,
            JournalPriority::Error     => Self::Error,
            JournalPriority::Warning   => Self::Warning,
            JournalPriority::Notice    => Self::Notice,
            JournalPriority::Info      => Self::Info,
            JournalPriority::Debug     => Self::Debug,
        }
    }
}

// ── JournalEntry → proto JournalsEntry ───────────────────────────────────────

impl From<JournalEntry> for JournalsEntry {
    fn from(e: JournalEntry) -> Self {
        Self {
            message:      e.message,
            timestamp_us: e.timestamp_us,
            priority:     Priority::from(e.priority) as i32,
            unit:         e.unit,
            cursor:       e.cursor,
            extra_fields: e.extra_fields.into_iter().collect(),
        }
    }
}

// ── JournalPage → proto GetJournalsResponse ───────────────────────────────────

impl From<JournalPage> for GetJournalsResponse {
    fn from(page: JournalPage) -> Self {
        Self {
            entries:    page.entries.into_iter().map(JournalsEntry::from).collect(),
            pagination: Some(PageResponse::from(page.pagination)),
        }
    }
}
