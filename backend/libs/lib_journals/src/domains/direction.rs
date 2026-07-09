//! Page direction indicator for journal seek operations.
//!
//! A [`Direction`] carries a single boolean flag indicating which way a
//! paginated seek should traverse the journal. Exactly one field is `true`
//! at any time; [`Direction::default`] begins in the `unspecified` state
//! until a caller selects a concrete direction.

/// Indicates the direction of a paginated journal seek.
///
/// Exactly one field should be `true` at a time. The [`Default`]
/// implementation sets [`unspecified`](Self::unspecified) to signal that
/// no direction has been chosen yet.
///
/// # Examples
///
/// ```
/// use lib_journals::domains::Direction;
///
/// let dir = Direction { forward: true, ..Direction::default() };
/// assert!(dir.forward);
/// assert!(!dir.unspecified);
/// ```
pub struct Direction {
    /// No direction has been specified; the seek direction is unresolved.
    pub unspecified: bool,

    /// Seek forward through the journal (oldest-to-newest).
    pub forward: bool,

    /// Seek backward through the journal (newest-to-oldest).
    pub backward: bool,
}

impl Default for Direction {
    /// Returns a `Direction` with no direction chosen (`unspecified = true`).
    fn default() -> Self {
        Self {
            unspecified: true,
            forward: false,
            backward: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sets_unspecified() {
        let dir = Direction::default();
        assert!(dir.unspecified);
        assert!(!dir.forward);
        assert!(!dir.backward);
    }

    #[test]
    fn forward_direction() {
        let dir = Direction {
            unspecified: false,
            forward: true,
            backward: false,
        };
        assert!(dir.forward);
        assert!(!dir.unspecified);
        assert!(!dir.backward);
    }

    #[test]
    fn backward_direction() {
        let dir = Direction {
            unspecified: false,
            forward: false,
            backward: true,
        };
        assert!(dir.backward);
        assert!(!dir.unspecified);
        assert!(!dir.forward);
    }
}
