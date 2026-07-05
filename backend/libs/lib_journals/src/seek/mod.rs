use crate::{JournalConnection, JournalPage, JournalQuery, Result};
use lib_core::domains::pagination::Direction;

mod backward;
mod forward;
// mod next;
// mod previous;

impl<'a> JournalQuery<'a> {
    pub fn seek(&self, conn: &mut JournalConnection) -> Result<JournalPage> {
        //--- Apply unit filter before reading any entries.
        if !self.unit_name.is_empty() {
            conn.match_unit(self.unit_name)?;
        }

        //--- Seek forward or backward based on the pagination direction.
        match self.pagination.direction {
            Direction::Forward => self.seek_forward(conn),
            Direction::Backward => self.seek_backward(conn),
        }
    }
}
