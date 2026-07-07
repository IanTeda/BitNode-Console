//! FollowJournals handler for the Journals gRPC service.

use std::pin::Pin;

use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};

use super::{FollowJournalsRequest, JournalsEntry};

/// Pinned boxed stream of journal entries yielded by [`handle`].
pub(super) type JournalStream =
    Pin<Box<dyn Stream<Item = std::result::Result<JournalsEntry, tonic::Status>> + Send>>;

/// Handle a `FollowJournals` request — stream live journal log entries.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    unit_name: &str,
    request: tonic::Request<FollowJournalsRequest>,
) -> crate::Result<tonic::Response<JournalStream>> {
    tracing::debug!("FollowJournals request from {:?}", request.remote_addr());

    let journal_query: lib_journals::JournalFollowTail = request.into_inner().into();

    // Clone fields before moving into the blocking thread.  `unit_name` comes
    // from service configuration (`&self.unit_name`) so it can't be `'static`
    // — owning it here lets the thread construct a fresh `JournalFollowTail`
    // that borrows from its own stack frame.
    let unit_name = unit_name.to_string();
    let priority = journal_query.priority;
    let tail_lines = journal_query.tail_lines;

    // Create a channel for streaming journal entries back to the caller.
    let (tx, rx) = mpsc::channel(16);

    // `follow` blocks the thread indefinitely (it calls `journal.wait(None)`
    // between batches), so it must run on a dedicated OS thread rather than a
    // Tokio worker.  `blocking_send` bridges back into the async world.
    std::thread::spawn(move || {
        let query = lib_journals::JournalFollowTail::new(unit_name.as_str(), priority, tail_lines);

        let mut conn = match lib_journals::JournalConnection::open_current_user() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.blocking_send(Err(crate::Error::Journal(e).into()));
                return;
            },
        };

        if let Err(e) = query.follow(&mut conn, |entry| {
            // Stop streaming when the receiver has been dropped (client
            // disconnected or the stream was cancelled).
            tx.blocking_send(Ok(entry.into())).is_ok()
        }) {
            let _ = tx.blocking_send(Err(crate::Error::Journal(e).into()));
        }
    });

    Ok(tonic::Response::new(Box::pin(ReceiverStream::new(rx))))
}
