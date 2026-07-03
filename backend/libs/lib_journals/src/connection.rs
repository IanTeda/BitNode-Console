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
    // /// Open the system journal.
    // ///
    // /// # Errors
    // ///
    // /// Returns [`Error::Io`] if the journal cannot be opened.
    // pub fn open(unit: &str) -> Result<Self> {
    //     let mut journal = OpenOptions::default().open()?;
    //     journal.match_add("_SYSTEMD_UNIT", unit)?;
    //     Ok(Self { journal })
    // }

    /// Open only the current user's journal slice.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the journal cannot be opened.
    pub fn open_current_user() -> Result<Self> {
        let journal = OpenOptions::default().current_user(true).open()?;
        Ok(Self { journal })
    }

    /// Add a `_SYSTEMD_UNIT` match filter so only entries for `unit` are returned.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the match cannot be applied.
    pub fn match_unit(&mut self, unit: &str) -> Result<()> {
        self.journal.match_add("_SYSTEMD_UNIT", unit)?;
        Ok(())
    }
}

impl std::fmt::Debug for JournalConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JournalConnection").finish_non_exhaustive()
    }
}
