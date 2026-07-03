//! Cursor-based pagination domain types.
//!
//! These types are transport-agnostic. Crates that speak proto (e.g. `lib_rpc`)
//! hold the `From` impls that convert to/from the generated proto messages.

/// Server-chosen page size when the client sends `page_size = 0`.
pub const DEFAULT_PAGE_SIZE: u32 = 50;

/// Direction controls which side of the pagination cursor to read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Read items after the cursor (chronologically newer for logs).
    #[default]
    Forward,
    /// Read items before the cursor (chronologically older for logs).
    Backward,
}

/// Cursor-based pagination parameters for a single page request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationRequest {
    /// Maximum number of items to return.
    ///
    /// Falls back to [`DEFAULT_PAGE_SIZE`] when the client sends zero.
    pub page_size: u32,

    /// Opaque cursor marking the page boundary; `None` starts from the natural edge.
    pub page_token: Option<String>,

    /// Which side of `page_token` to read from.
    pub direction: Direction,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page_size: DEFAULT_PAGE_SIZE,
            page_token: None,
            direction: Direction::default(),
        }
    }
}

/// Cursor-based pagination metadata returned alongside a page of results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaginationResponse {
    /// Cursor for the next page (newer), absent when `has_next_page` is false.
    pub next_page_token: Option<String>,

    /// Cursor for the previous page (older), absent when `has_prev_page` is false.
    pub prev_page_token: Option<String>,

    /// True when at least one more page exists in the forward direction.
    pub has_next_page: bool,

    /// True when at least one more page exists in the backward direction.
    pub has_prev_page: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_defaults_to_forward() {
        assert_eq!(Direction::default(), Direction::Forward);
    }

    #[test]
    fn page_request_default_uses_default_page_size() {
        assert_eq!(PaginationRequest::default().page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_request_default_has_no_token() {
        assert!(PaginationRequest::default().page_token.is_none());
    }

    #[test]
    fn page_request_default_direction_is_forward() {
        assert_eq!(PaginationRequest::default().direction, Direction::Forward);
    }

    #[test]
    fn page_response_default_has_no_tokens() {
        let r = PaginationResponse::default();
        assert!(r.next_page_token.is_none());
        assert!(r.prev_page_token.is_none());
        assert!(!r.has_next_page);
        assert!(!r.has_prev_page);
    }
}
