//! Persistent systemd journal connection.

use systemd::journal::{Journal, OpenOptions};

use crate::{Error, Result};

/// A persistent open handle to the systemd journal.
///
/// Use [`JournalConnection::open`] to create a system-journal connection, or
/// [`JournalConnection::open_current_user`] when only the calling user's
/// journal slice is required.  Add match filters with
/// [`match_add`][Self::match_add] before iterating or streaming entries.
pub struct JournalConnection {
    pub(crate) journal: Journal,
}

impl JournalConnection {
    /// Open the system journal.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the journal cannot be opened.
    pub fn open(unit: &str) -> Result<Self> {
        let mut journal = OpenOptions::default().open()?;
        journal.match_add("_SYSTEMD_UNIT", unit)?;
        Ok(Self { journal })
    }

    /// Open only the current user's journal slice.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the journal cannot be opened.
    pub fn open_current_user(unit: &str) -> Result<Self> {
        let mut journal = OpenOptions::default().current_user(true).open()?;
        journal.match_add("_SYSTEMD_UNIT", unit)?;
        Ok(Self { journal })
    }
}
