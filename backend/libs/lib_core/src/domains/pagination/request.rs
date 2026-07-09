use crate::domains::pagination::Direction;

/// Server-chosen page size when the client sends `page_size = 0`.
pub const DEFAULT_PAGE_SIZE: u32 = 50;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
