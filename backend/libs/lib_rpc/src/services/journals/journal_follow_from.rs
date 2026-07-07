//! Conversion from [`FollowJournalsRequest`] into [`JournalFollowTail`].

use lib_journals::{JournalFollowTail, JournalPriority};

use crate::generated_protos::journals::{FollowJournalsRequest, Priority};

// ── FollowJournalsRequest → JournalFollowTail ─────────────────────────────────

impl From<FollowJournalsRequest> for JournalFollowTail<'static> {
    /// Converts a gRPC [`FollowJournalsRequest`] into a domain [`JournalFollowTail`].
    ///
    /// `unit_name` is not part of the request — it is fixed by server configuration.
    /// The returned query uses the empty-string default; callers must supply the real
    /// unit name from settings before dispatching, e.g.:
    ///
    /// ```rust,ignore
    /// let mut query = JournalFollowTail::from(req);
    /// query.unit_name = &settings.unit_name;
    /// ```
    ///
    /// A `tail_lines` value of `0` (the protobuf default) falls back to the domain
    /// type's own default.
    fn from(req: FollowJournalsRequest) -> Self {
        let priority = req
            .priority
            .and_then(|i| Priority::try_from(i).ok())
            .map(JournalPriority::from)
            .unwrap_or(JournalPriority::Info);

        let tail_lines = req
            .tail_lines
            .filter(|&n| n > 0)
            .unwrap_or_else(|| JournalFollowTail::default().tail_lines);

        Self {
            unit_name: "",
            priority,
            tail_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_req(tail_lines: Option<u32>, priority: Option<i32>) -> FollowJournalsRequest {
        FollowJournalsRequest { tail_lines, priority }
    }

    #[test]
    fn priority_maps_correctly() {
        let cases = [
            (Priority::Unspecified as i32, JournalPriority::Info),
            (Priority::Emergency   as i32, JournalPriority::Emergency),
            (Priority::Alert       as i32, JournalPriority::Alert),
            (Priority::Critical    as i32, JournalPriority::Critical),
            (Priority::Error       as i32, JournalPriority::Error),
            (Priority::Warning     as i32, JournalPriority::Warning),
            (Priority::Notice      as i32, JournalPriority::Notice),
            (Priority::Info        as i32, JournalPriority::Info),
            (Priority::Debug       as i32, JournalPriority::Debug),
        ];
        for (proto, expected) in cases {
            let q = JournalFollowTail::from(make_req(Some(10), Some(proto)));
            assert_eq!(q.priority, expected, "priority mismatch for proto value {proto}");
        }
    }

    #[test]
    fn absent_priority_defaults_to_info() {
        let q = JournalFollowTail::from(make_req(Some(10), None));
        assert_eq!(q.priority, JournalPriority::Info);
    }

    #[test]
    fn invalid_priority_falls_back_to_info() {
        let q = JournalFollowTail::from(make_req(Some(10), Some(999)));
        assert_eq!(q.priority, JournalPriority::Info);
    }

    #[test]
    fn tail_lines_passes_through() {
        let q = JournalFollowTail::from(make_req(Some(50), None));
        assert_eq!(q.tail_lines, 50);
    }

    #[test]
    fn absent_tail_lines_uses_domain_default() {
        let default_lines = JournalFollowTail::default().tail_lines;
        let q = JournalFollowTail::from(make_req(None, None));
        assert_eq!(q.tail_lines, default_lines);
    }

    #[test]
    fn zero_tail_lines_uses_domain_default() {
        let default_lines = JournalFollowTail::default().tail_lines;
        let q = JournalFollowTail::from(make_req(Some(0), None));
        assert_eq!(q.tail_lines, default_lines);
    }

    #[test]
    fn unit_name_is_empty() {
        let q = JournalFollowTail::from(make_req(Some(10), None));
        assert_eq!(q.unit_name, "");
    }
}
