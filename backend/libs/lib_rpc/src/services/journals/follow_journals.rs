//! `FollowJournals` handler for the Journals gRPC service.
//!
//! Streams live journal entries for a configured systemd unit over a bounded
//! `mpsc` channel.  The blocking `journal.wait(None)` call runs on a dedicated
//! OS thread so it never stalls a Tokio worker.  Errors from the journal are
//! forwarded through the stream as [`tonic::Status::internal`] rather than
//! returned directly from [`handle`], so the channel is always created and a
//! stream is always returned to the caller.

use std::pin::Pin;

use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};

use super::{FollowJournalsRequest, JournalsEntry};

/// Pinned, heap-allocated stream of [`JournalsEntry`] items yielded to tonic.
///
/// Each item is either a successfully converted journal entry or a
/// [`tonic::Status`] error forwarded from the background thread.
pub(super) type JournalStream =
    Pin<Box<dyn Stream<Item = std::result::Result<JournalsEntry, tonic::Status>> + Send>>;

/// Handle a `FollowJournals` RPC — open the journal and stream live entries.
///
/// `unit_name` comes from the service configuration (e.g. `"bitcoind.service"`);
/// it is not part of the client request.
///
/// # Threading model
///
/// `lib_journals::FollowTail::follow` blocks the calling thread indefinitely
/// between journal events, so the entire read loop runs on a dedicated OS
/// thread (via `std::thread::spawn`).  Each entry is sent back into async
/// land through a bounded `mpsc` channel (capacity 16), which provides
/// implicit backpressure: the thread blocks on `blocking_send` whenever the
/// consumer falls behind.
///
/// # Cancellation
///
/// When the client disconnects (or the returned stream is dropped), the channel
/// receiver is dropped.  The next `blocking_send` inside the callback then
/// returns `Err`, which causes `follow` to return early via the `false` signal.
/// If the thread is already blocked inside `journal.wait(None)`, it will not
/// wake until the next matching journal event arrives.
///
/// # Errors
///
/// Journal errors (connection failure, I/O) are sent as `Err(tonic::Status)`
/// items through the stream rather than returned directly; `handle` itself is
/// infallible once the channel is set up.
#[tracing::instrument(skip(request))]
pub(super) async fn handle(
    unit_name: &str,
    request: tonic::Request<FollowJournalsRequest>,
) -> crate::Result<tonic::Response<JournalStream>> {
    tracing::debug!("FollowJournals request from {:?}", request.remote_addr());

    let journal_query: lib_journals::FollowTail = request.into_inner().into();

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
        let query = lib_journals::FollowTail::new(unit_name.as_str(), priority, tail_lines);

        let mut conn = match lib_journals::Connection::open_current_user() {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio_stream::StreamExt as _;

    use super::*;
    use crate::generated_protos::journals::FollowJournalsRequest;

    fn follow_request(
        tail_lines: Option<u32>,
        priority: Option<i32>,
    ) -> tonic::Request<FollowJournalsRequest> {
        tonic::Request::new(FollowJournalsRequest { tail_lines, priority })
    }

    /// `handle` always returns `Ok` — journal errors surface as stream items,
    /// not as the direct return value.
    #[tokio::test]
    async fn handle_always_returns_ok() {
        let result = handle("bitcoind.service", follow_request(Some(10), None)).await;
        assert!(result.is_ok());
    }

    /// The response wraps a valid, pinned stream that can be unwrapped without
    /// panicking — confirming the channel and type-boxing steps succeed.
    #[tokio::test]
    async fn response_contains_valid_stream() {
        let response = handle("bitcoind.service", follow_request(Some(10), None))
            .await
            .unwrap();
        let _stream = response.into_inner();
    }

    /// Dropping the stream causes the channel receiver to close.  The background
    /// thread's next `blocking_send` returns `Err`, which is mapped to `false`
    /// by `is_ok()`, stopping the follow loop cleanly.
    #[tokio::test]
    async fn dropping_stream_does_not_panic() {
        let response = handle("bitcoind.service", follow_request(Some(1), None))
            .await
            .unwrap();
        drop(response);
    }

    /// When the unit produces no matching entries, the stream stays open but
    /// never yields within the timeout — verifying the filter is applied and
    /// the thread does not spin.
    #[tokio::test]
    async fn stream_does_not_yield_for_nonexistent_unit() {
        let response = handle(
            "__nonexistent_unit__.service",
            follow_request(Some(1), None),
        )
        .await
        .unwrap();

        let mut stream = response.into_inner();

        let timed =
            tokio::time::timeout(Duration::from_millis(200), stream.next()).await;
        assert!(timed.is_err(), "expected timeout for a unit with no entries");
    }
}
