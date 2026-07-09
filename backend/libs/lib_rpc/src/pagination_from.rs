//! Proto ↔ domain pagination conversion impls.
//!
//! Domain types live in [`lib_core::domains::pagination`]; this module only
//! holds the conversions that require access to the generated proto types.

use crate::services::pagination::{PageDirection, PageRequest, PageResponse};

pub use lib_core::domains::pagination::{
    DEFAULT_PAGE_SIZE, Direction, PaginationRequest, PaginationResponse,
};

// --- proto Direction ↔ domain Direction ------------------------------------------

impl From<PageDirection> for Direction {
    fn from(proto: PageDirection) -> Self {
        match proto {
            PageDirection::Forward => Self::Forward,
            PageDirection::Backward => Self::Backward,
            // Unspecified defaults to forward per the proto definition.
            PageDirection::Unspecified => Self::Forward,
        }
    }
}

impl From<Direction> for PageDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Forward => Self::Forward,
            Direction::Backward => Self::Backward,
        }
    }
}

// --- proto PageRequest → domain PaginationRequest --------------------------------

impl From<PageRequest> for PaginationRequest {
    fn from(proto: PageRequest) -> Self {
        let direction = PageDirection::try_from(proto.page_direction)
            .map(Direction::from)
            .unwrap_or(Direction::Forward);
        Self {
            page_size: if proto.page_size == 0 {
                DEFAULT_PAGE_SIZE
            } else {
                proto.page_size
            },
            page_token: proto.page_token,
            direction,
        }
    }
}

// --- domain PaginationResponse → proto PageResponse ------------------------------

impl From<PaginationResponse> for PageResponse {
    fn from(page: PaginationResponse) -> Self {
        Self {
            page_token_next: page.next_page_token,
            page_token_prev: page.prev_page_token,
        }
    }
}

// ---------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Direction conversions ---------------------------------------------------

    #[test]
    fn proto_forward_converts_to_domain_forward() {
        assert_eq!(Direction::from(PageDirection::Forward), Direction::Forward);
    }

    #[test]
    fn proto_backward_converts_to_domain_backward() {
        assert_eq!(
            Direction::from(PageDirection::Backward),
            Direction::Backward
        );
    }

    #[test]
    fn proto_unspecified_direction_defaults_to_forward() {
        assert_eq!(
            Direction::from(PageDirection::Unspecified),
            Direction::Forward
        );
    }

    #[test]
    fn domain_forward_converts_to_proto_forward() {
        assert_eq!(
            PageDirection::from(Direction::Forward),
            PageDirection::Forward
        );
    }

    #[test]
    fn domain_backward_converts_to_proto_backward() {
        assert_eq!(
            PageDirection::from(Direction::Backward),
            PageDirection::Backward
        );
    }

    // --- PageRequest conversion --------------------------------------------------

    #[test]
    fn page_request_fields_convert_from_proto() {
        let proto = PageRequest {
            page_size: 25,
            page_token: Some("cursor_abc".to_string()),
            page_direction: PageDirection::Forward as i32,
        };
        let req = PaginationRequest::from(proto);
        assert_eq!(req.page_size, 25);
        assert_eq!(req.page_token.as_deref(), Some("cursor_abc"));
        assert_eq!(req.direction, Direction::Forward);
    }

    #[test]
    fn page_request_zero_page_size_falls_back_to_default() {
        let proto = PageRequest {
            page_size: 0,
            page_token: None,
            page_direction: PageDirection::Unspecified as i32,
        };
        assert_eq!(PaginationRequest::from(proto).page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_request_unknown_direction_i32_defaults_to_forward() {
        let proto = PageRequest {
            page_size: 10,
            page_token: None,
            page_direction: 99,
        };
        assert_eq!(PaginationRequest::from(proto).direction, Direction::Forward);
    }

    // --- PageResponse conversion -------------------------------------------------

    #[test]
    fn page_response_tokens_convert_to_proto() {
        let domain = PaginationResponse {
            next_page_token: Some("next".to_string()),
            prev_page_token: Some("prev".to_string()),
        };
        let proto = PageResponse::from(domain);
        assert_eq!(proto.page_token_next.as_deref(), Some("next"));
        assert_eq!(proto.page_token_prev.as_deref(), Some("prev"));
    }

    #[test]
    fn default_page_response_converts_to_empty_proto() {
        let proto = PageResponse::from(PaginationResponse::default());
        assert!(proto.page_token_next.is_none());
        assert!(proto.page_token_prev.is_none());
    }
}
