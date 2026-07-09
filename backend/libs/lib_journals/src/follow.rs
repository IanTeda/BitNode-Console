//! Journal follow-tail — applies unit filter, seeks to tail, then streams live entries.

use systemd::journal::JournalSeek;

use crate::{Connection, Entry, FollowTail, Result};

impl FollowTail<'_> {
    /// Stream journal entries for the configured unit in chronological order.
    ///
    /// Replays the last [`JournalFollowTail::tail_lines`] entries then blocks,
    /// waking whenever the journal appends new entries that match the configured
    /// filters.
    ///
    /// `on_entry` is called for each matched entry.  Return `false` to stop the
    /// stream; return `true` to continue.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if any underlying journal operation fails.
    pub fn follow<F>(&self, conn: &mut Connection, mut on_entry: F) -> Result<()>
    where
        F: FnMut(Entry) -> bool,
    {
        // Apply unit filter before reading any entries.
        if !self.unit_name.is_empty() {
            conn.match_unit(self.unit_name)?;
        }

        // --- 01. Seek to tail and step back to replay recent history.
        //
        // The +1 compensates for next_entry() advancing the cursor once before
        // reading, which would otherwise skip the entry we land on after
        // previous_skip().
        conn.journal.seek(JournalSeek::Tail)?;
        conn.journal.previous_skip(u64::from(self.tail_lines) + 1)?;

        // --- 02. Stream entries, blocking between batches.
        loop {
            // Drain all currently available entries from the current position.
            while let Some(record) = conn.journal.next_entry()? {
                let timestamp_us = i64::try_from(conn.journal.timestamp_usec()?)?;

                let mut entry = Entry::from_record(&record, timestamp_us);
                entry.cursor = conn.journal.cursor().ok();

                if entry.priority <= self.priority && !on_entry(entry) {
                    return Ok(());
                }
            }

            // Block until the journal signals that new entries are available.
            conn.journal.wait(None)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Connection, FollowTail, Priority};

    /// Inject a line into the system journal under `identifier` at the given syslog priority.
    fn inject(identifier: &str, priority: &str, message: &str) {
        use std::io::Write as _;
        let mut child = std::process::Command::new("systemd-cat")
            .args(["-t", identifier, "-p", priority])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("systemd-cat must be available");
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(message.as_bytes()).ok();
        }
        child.wait().expect("systemd-cat must complete");
    }

    fn open() -> Connection {
        Connection::open().expect("system journal must open")
    }

    #[test]
    fn replays_tail_lines_before_blocking() {
        let id = "lib-jd-follow-tail-replay";
        for i in 0..3 {
            inject(id, "info", &format!("replay-msg-{i}"));
        }

        let mut conn = open();
        let mut collected = Vec::new();
        let query = FollowTail::with_unit("lib-jd-follow-tail-replay.service");

        query
            .follow(&mut conn, |entry| {
                collected.push(entry.message);
                collected.len() < 3
            })
            .expect("follow must succeed");

        assert!(
            !collected.is_empty(),
            "expected at least one replayed entry before blocking"
        );
    }

    #[test]
    fn callback_returning_false_stops_stream() {
        let id = "lib-jd-follow-tail-stop";
        for i in 0..5 {
            inject(id, "info", &format!("stop-msg-{i}"));
        }

        let mut conn = open();
        let mut count = 0usize;
        let query = FollowTail::with_unit("lib-jd-follow-tail-stop.service");

        query
            .follow(&mut conn, |_entry| {
                count += 1;
                count < 2
            })
            .expect("follow must succeed");

        assert_eq!(count, 2, "stream must stop after callback returns false");
    }

    #[test]
    fn priority_threshold_filters_less_severe_entries() {
        let id = "lib-jd-follow-tail-priority";
        inject(id, "info", "info-msg-should-be-excluded");

        let mut conn = open();
        let query = FollowTail::new("lib-jd-follow-tail-priority.service", Priority::Error, 5);

        let mut received_any = false;
        query
            .follow(&mut conn, |entry| {
                received_any = true;
                assert!(
                    entry.priority <= Priority::Error,
                    "entry with priority {:?} must not exceed Error threshold",
                    entry.priority,
                );
                false
            })
            .expect("follow must succeed");

        assert!(
            !received_any,
            "no error-priority entries should have been yielded"
        );
    }
}
