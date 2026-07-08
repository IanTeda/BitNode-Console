//! Bidirectional conversion between [`lib_journals::Priority`] and the
//! protobuf [`Priority`] enum.
//!
//! **Outbound** (`lib_journals::Priority` → [`Priority`]): used when building
//! a [`JournalsEntry`] response to send to gRPC clients.  All eight syslog
//! severity levels have a one-to-one proto correspondent, so the mapping is
//! exhaustive and lossless.
//!
//! **Inbound** ([`Priority`] → `lib_journals::Priority`): used when reading a
//! priority filter from a client request.  `Priority::Unspecified` (the proto
//! wire default `0`, sent when the client omits the field) maps to
//! [`lib_journals::Priority::Info`].

use crate::generated_protos::journals::Priority;

/// Converts a domain [`lib_journals::Priority`] to its proto counterpart.
///
/// The mapping is exhaustive and lossless: every domain variant has an exact
/// equivalent in the generated [`Priority`] enum.
impl From<lib_journals::Priority> for Priority {
    fn from(p: lib_journals::Priority) -> Self {
        match p {
            lib_journals::Priority::Emergency => Self::Emergency,
            lib_journals::Priority::Alert => Self::Alert,
            lib_journals::Priority::Critical => Self::Critical,
            lib_journals::Priority::Error => Self::Error,
            lib_journals::Priority::Warning => Self::Warning,
            lib_journals::Priority::Notice => Self::Notice,
            lib_journals::Priority::Info => Self::Info,
            lib_journals::Priority::Debug => Self::Debug,
        }
    }
}

/// Converts a proto [`Priority`] to its domain counterpart.
///
/// `Priority::Unspecified` (the zero-value wire default) is treated as
/// [`lib_journals::Priority::Info`], matching the behaviour of the other
/// request-conversion modules.
impl From<Priority> for lib_journals::Priority {
    fn from(p: Priority) -> Self {
        match p {
            Priority::Unspecified => Self::Info,
            Priority::Emergency => Self::Emergency,
            Priority::Alert => Self::Alert,
            Priority::Critical => Self::Critical,
            Priority::Error => Self::Error,
            Priority::Warning => Self::Warning,
            Priority::Notice => Self::Notice,
            Priority::Info => Self::Info,
            Priority::Debug => Self::Debug,
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_journals::Priority as Domain;

    use super::*;

    // ── domain → proto (outbound) ─────────────────────────────────────────────

    #[test]
    fn domain_to_proto_all_variants() {
        let cases = [
            (Domain::Emergency, Priority::Emergency),
            (Domain::Alert, Priority::Alert),
            (Domain::Critical, Priority::Critical),
            (Domain::Error, Priority::Error),
            (Domain::Warning, Priority::Warning),
            (Domain::Notice, Priority::Notice),
            (Domain::Info, Priority::Info),
            (Domain::Debug, Priority::Debug),
        ];
        for (domain, expected) in cases {
            assert_eq!(
                Priority::from(domain),
                expected,
                "unexpected mapping for {domain:?}",
            );
        }
    }

    // ── proto → domain (inbound) ──────────────────────────────────────────────

    #[test]
    fn proto_to_domain_all_named_variants() {
        let cases = [
            (Priority::Emergency, Domain::Emergency),
            (Priority::Alert, Domain::Alert),
            (Priority::Critical, Domain::Critical),
            (Priority::Error, Domain::Error),
            (Priority::Warning, Domain::Warning),
            (Priority::Notice, Domain::Notice),
            (Priority::Info, Domain::Info),
            (Priority::Debug, Domain::Debug),
        ];
        for (proto, expected) in cases {
            assert_eq!(
                Domain::from(proto),
                expected,
                "unexpected mapping for {proto:?}",
            );
        }
    }

    /// `Unspecified` is the proto wire default (`0`) sent when a client omits
    /// the priority field entirely; it must map to `Info`.
    #[test]
    fn proto_unspecified_maps_to_info() {
        assert_eq!(Domain::from(Priority::Unspecified), Domain::Info);
    }
}
