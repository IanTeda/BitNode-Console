//! Cursor-based pagination domain types and conversions to/from generated proto types.

use crate::generated_protos::common as proto;

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

/// Cursor-based pagination parameters decoded from an RPC request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRequest {
    /// Maximum number of items to return.
    ///
    /// Falls back to [`DEFAULT_PAGE_SIZE`] when the client sends zero.
    pub page_size: u32,

    /// Opaque cursor marking the page boundary; `None` starts from the natural edge.
    pub page_token: Option<String>,

    /// Which side of `page_token` to read from.
    pub direction: Direction,
}

/// Cursor-based pagination metadata returned alongside a page of results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageResponse {
    /// Cursor for the next page (newer), absent when `has_next_page` is false.
    pub next_page_token: Option<String>,

    /// Cursor for the previous page (older), absent when `has_prev_page` is false.
    pub prev_page_token: Option<String>,

    /// True when at least one more page exists in the forward direction.
    pub has_next_page: bool,

    /// True when at least one more page exists in the backward direction.
    pub has_prev_page: bool,
}

// --- proto Direction ↔ domain Direction ------------------------------------------

impl From<proto::Direction> for Direction {
    fn from(proto: proto::Direction) -> Self {
        match proto {
            proto::Direction::Forward => Self::Forward,
            proto::Direction::Backward => Self::Backward,
            // Unspecified defaults to forward per the proto definition.
            proto::Direction::Unspecified => Self::Forward,
        }
    }
}

impl From<Direction> for proto::Direction {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Forward => Self::Forward,
            Direction::Backward => Self::Backward,
        }
    }
}

// --- proto PageRequest → domain PageRequest ---------------------------------------

impl From<proto::PageRequest> for PageRequest {
    fn from(proto: proto::PageRequest) -> Self {
        // prost stores enum fields as i32; fall back to Unspecified on unknown values.
        let direction = proto::Direction::try_from(proto.direction)
            .unwrap_or(proto::Direction::Unspecified)
            .into();
        Self {
            page_size: if proto.page_size == 0 { DEFAULT_PAGE_SIZE } else { proto.page_size },
            page_token: proto.page_token,
            direction,
        }
    }
}

// --- domain PageResponse → proto PageResponse ------------------------------------

impl From<PageResponse> for proto::PageResponse {
    fn from(page: PageResponse) -> Self {
        Self {
            next_page_token: page.next_page_token,
            prev_page_token: page.prev_page_token,
            has_next_page: page.has_next_page,
            has_prev_page: page.has_prev_page,
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
        assert_eq!(Direction::from(proto::Direction::Forward), Direction::Forward);
    }

    #[test]
    fn proto_backward_converts_to_domain_backward() {
        assert_eq!(Direction::from(proto::Direction::Backward), Direction::Backward);
    }

    #[test]
    fn proto_unspecified_direction_defaults_to_forward() {
        assert_eq!(Direction::from(proto::Direction::Unspecified), Direction::Forward);
    }

    #[test]
    fn domain_forward_converts_to_proto_forward() {
        assert_eq!(proto::Direction::from(Direction::Forward), proto::Direction::Forward);
    }

    #[test]
    fn domain_backward_converts_to_proto_backward() {
        assert_eq!(proto::Direction::from(Direction::Backward), proto::Direction::Backward);
    }

    // --- PageRequest conversion --------------------------------------------------

    #[test]
    fn page_request_fields_convert_from_proto() {
        let proto = proto::PageRequest {
            page_size: 25,
            page_token: Some("cursor_abc".to_string()),
            direction: proto::Direction::Forward as i32,
        };
        let req = PageRequest::from(proto);
        assert_eq!(req.page_size, 25);
        assert_eq!(req.page_token.as_deref(), Some("cursor_abc"));
        assert_eq!(req.direction, Direction::Forward);
    }

    #[test]
    fn page_request_zero_page_size_falls_back_to_default() {
        let proto = proto::PageRequest {
            page_size: 0,
            page_token: None,
            direction: proto::Direction::Unspecified as i32,
        };
        assert_eq!(PageRequest::from(proto).page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_request_unknown_direction_i32_defaults_to_forward() {
        let proto = proto::PageRequest {
            page_size: 10,
            page_token: None,
            direction: 99,
        };
        assert_eq!(PageRequest::from(proto).direction, Direction::Forward);
    }

    // --- PageResponse conversion -------------------------------------------------

    #[test]
    fn page_response_converts_to_proto() {
        let domain = PageResponse {
            next_page_token: Some("next".to_string()),
            prev_page_token: Some("prev".to_string()),
            has_next_page: true,
            has_prev_page: false,
        };
        let proto = proto::PageResponse::from(domain);
        assert_eq!(proto.next_page_token.as_deref(), Some("next"));
        assert_eq!(proto.prev_page_token.as_deref(), Some("prev"));
        assert!(proto.has_next_page);
        assert!(!proto.has_prev_page);
    }

    #[test]
    fn default_page_response_converts_to_empty_proto() {
        let proto = proto::PageResponse::from(PageResponse::default());
        assert!(proto.next_page_token.is_none());
        assert!(proto.prev_page_token.is_none());
        assert!(!proto.has_next_page);
        assert!(!proto.has_prev_page);
    }
}
