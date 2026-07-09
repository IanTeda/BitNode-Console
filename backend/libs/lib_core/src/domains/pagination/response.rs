/// Cursor-based pagination metadata returned alongside a page of results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaginationResponse {
    /// Cursor for the next page (newer), absent when `has_next_page` is false.
    pub next_page_token: Option<String>,

    /// Cursor for the previous page (older), absent when `has_prev_page` is false.
    pub prev_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_response_default_has_no_tokens() {
        let r = PaginationResponse::default();
        assert!(r.next_page_token.is_none());
        assert!(r.prev_page_token.is_none());
    }
}
