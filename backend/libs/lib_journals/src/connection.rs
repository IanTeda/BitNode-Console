//! Persistent systemd journal connection.

use systemd::journal::{Journal, OpenOptions};

use crate::{Error, Result};

/// A persistent open handle to the systemd journal.
///
/// Use [`JournalConnection::open`] to create a system-journal connection, or
/// [`JournalConnection::open_current_user`] when only the calling user's
/// journal slice is required.  Add match filters with
/// [`match_add`][Self::match_add] before iterating or streaming entries.
pub struct Connection {
    pub(crate) journal: Journal,
}

impl Connection {
    /// Open the system journal.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the journal cannot be opened.
    pub fn open() -> Result<Self> {
        let journal = OpenOptions::default().open()?;
        Ok(Self { journal })
    }

    /// Open only the current user's journal slice.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the journal cannot be opened.
    pub fn open_current_user() -> Result<Self> {
        let journal = OpenOptions::default().current_user(true).open()?;
        Ok(Self { journal })
    }

    /// Add match filters so only entries for `unit` are returned.
    ///
    /// Matches `_SYSTEMD_UNIT=<unit>` OR `SYSLOG_IDENTIFIER=<unit-without-.service>`,
    /// covering both real systemd units and the `systemd-cat -t` dev workflow.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if any match cannot be applied.
    pub fn match_unit(&mut self, unit: &str) -> Result<()> {
        self.journal.match_add("_SYSTEMD_UNIT", unit)?;
        if let Some(syslog_id) = unit.strip_suffix(".service") {
            self.journal.match_or()?;
            self.journal.match_add("SYSLOG_IDENTIFIER", syslog_id)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JournalConnection").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_returns_ok() {
        assert!(Connection::open().is_ok());
    }

    #[test]
    fn open_current_user_returns_ok() {
        assert!(Connection::open_current_user().is_ok());
    }

    #[test]
    fn debug_contains_type_name() {
        let conn = Connection::open().unwrap();
        assert!(format!("{conn:?}").contains("JournalConnection"));
    }

    #[test]
    fn match_unit_with_service_suffix_returns_ok() {
        let mut conn = Connection::open().unwrap();
        assert!(conn.match_unit("bitcoind.service").is_ok());
    }

    #[test]
    fn match_unit_without_service_suffix_returns_ok() {
        let mut conn = Connection::open().unwrap();
        assert!(conn.match_unit("kernel").is_ok());
    }
}
