//! Conversion from [`FollowJournalsRequest`] into [`lib_journals::FollowTail`].
//!
//! Three mapping rules apply:
//!
//! * **`unit_name`** — not present in the request; the returned struct always
//!   carries an empty-string placeholder.  Callers must substitute the real
//!   unit name (from server configuration) before dispatching.
//! * **`priority`** — `None`, `Unspecified` (proto default `0`), or an
//!   unrecognised `i32` all fall back to [`lib_journals::Priority::Info`].
//! * **`tail_lines`** — `None` or `0` (the protobuf wire default) fall back to
//!   [`lib_journals::FollowTail::default`]'s tail-line count.

use crate::generated_protos::journals::{FollowJournalsRequest, Priority};

/// Converts a gRPC [`FollowJournalsRequest`] into a domain [`lib_journals::FollowTail`].
///
/// `unit_name` is not part of the request — it is fixed by server configuration.
/// The returned query carries an empty-string placeholder; callers must
/// substitute the real value before dispatching.  The canonical pattern used
/// by the `FollowJournals` handler is to extract the converted fields and pass
/// them to [`lib_journals::FollowTail::new`]:
///
/// ```rust,ignore
/// let q: lib_journals::FollowTail = request.into_inner().into();
/// let query = lib_journals::FollowTail::new(&unit_name, q.priority, q.tail_lines);
/// ```
///
/// A `tail_lines` value of `0` (the protobuf wire default) falls back to the
/// domain type's own default.  An invalid `priority` `i32` that
/// [`Priority::try_from`] cannot parse is also treated as `Info`.
impl From<FollowJournalsRequest> for lib_journals::FollowTail<'static> {
    fn from(req: FollowJournalsRequest) -> Self {
        let priority = req
            .priority
            .and_then(|i| Priority::try_from(i).ok())
            .map(|p| match p {
                Priority::Emergency => lib_journals::Priority::Emergency,
                Priority::Alert => lib_journals::Priority::Alert,
                Priority::Critical => lib_journals::Priority::Critical,
                Priority::Error => lib_journals::Priority::Error,
                Priority::Warning => lib_journals::Priority::Warning,
                Priority::Notice => lib_journals::Priority::Notice,
                Priority::Debug => lib_journals::Priority::Debug,
                Priority::Unspecified | Priority::Info => lib_journals::Priority::Info,
            })
            .unwrap_or(lib_journals::Priority::Info);

        let tail_lines = req
            .tail_lines
            .filter(|&n| n > 0)
            .unwrap_or_else(|| lib_journals::FollowTail::default().tail_lines);

        Self {
            unit_name: "",
            priority,
            tail_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_journals::{FollowTail, Priority as DomainPriority};

    use super::*;

    fn make_req(tail_lines: Option<u32>, priority: Option<i32>) -> FollowJournalsRequest {
        FollowJournalsRequest { tail_lines, priority }
    }

    /// Every proto `Priority` variant maps to its domain counterpart;
    /// `Unspecified` (the proto zero-value default) maps to `Info`.
    #[test]
    fn priority_maps_correctly() {
        let cases = [
            (Priority::Unspecified as i32, DomainPriority::Info),
            (Priority::Emergency as i32, DomainPriority::Emergency),
            (Priority::Alert as i32, DomainPriority::Alert),
            (Priority::Critical as i32, DomainPriority::Critical),
            (Priority::Error as i32, DomainPriority::Error),
            (Priority::Warning as i32, DomainPriority::Warning),
            (Priority::Notice as i32, DomainPriority::Notice),
            (Priority::Info as i32, DomainPriority::Info),
            (Priority::Debug as i32, DomainPriority::Debug),
        ];
        for (proto, expected) in cases {
            let q = FollowTail::from(make_req(Some(10), Some(proto)));
            assert_eq!(q.priority, expected, "priority mismatch for proto value {proto}");
        }
    }

    #[test]
    fn absent_priority_defaults_to_info() {
        let q = FollowTail::from(make_req(Some(10), None));
        assert_eq!(q.priority, DomainPriority::Info);
    }

    #[test]
    fn invalid_priority_falls_back_to_info() {
        let q = FollowTail::from(make_req(Some(10), Some(999)));
        assert_eq!(q.priority, DomainPriority::Info);
    }

    #[test]
    fn tail_lines_passes_through() {
        let q = FollowTail::from(make_req(Some(50), None));
        assert_eq!(q.tail_lines, 50);
    }

    #[test]
    fn absent_tail_lines_uses_domain_default() {
        let default_lines = FollowTail::default().tail_lines;
        let q = FollowTail::from(make_req(None, None));
        assert_eq!(q.tail_lines, default_lines);
    }

    #[test]
    fn zero_tail_lines_uses_domain_default() {
        let default_lines = FollowTail::default().tail_lines;
        let q = FollowTail::from(make_req(Some(0), None));
        assert_eq!(q.tail_lines, default_lines);
    }

    /// `unit_name` is always the empty-string placeholder regardless of other
    /// fields — the real value is injected by the handler after conversion.
    #[test]
    fn unit_name_is_always_empty() {
        let q = FollowTail::from(make_req(Some(10), Some(Priority::Warning as i32)));
        assert_eq!(q.unit_name, "");
    }

    /// A fully-populated request produces the expected struct with all three
    /// fields correctly set.
    #[test]
    fn fully_populated_request_converts_all_fields() {
        let q = FollowTail::from(make_req(Some(25), Some(Priority::Warning as i32)));
        assert_eq!(q.unit_name, "");
        assert_eq!(q.priority, DomainPriority::Warning);
        assert_eq!(q.tail_lines, 25);
    }
}
