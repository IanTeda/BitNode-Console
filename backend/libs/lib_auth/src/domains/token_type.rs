//! Token type discriminant for JSON Web Tokens.

/// Distinguishes access tokens from refresh tokens.
///
/// An access token authorises a single RPC request. A refresh token authorises
/// the issuance of a new access token and has a longer validity window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TokenType {
    #[default]
    Access,
    Refresh,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Access => write!(f, "Access"),
            TokenType::Refresh => write!(f, "Refresh"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenType;

    #[test]
    fn default_is_access() {
        assert_eq!(TokenType::default(), TokenType::Access);
    }

    #[test]
    fn access_displays_as_access() {
        assert_eq!(TokenType::Access.to_string(), "Access");
    }

    #[test]
    fn refresh_displays_as_refresh() {
        assert_eq!(TokenType::Refresh.to_string(), "Refresh");
    }

    #[test]
    fn display_and_to_string_agree() {
        for variant in [&TokenType::Access, &TokenType::Refresh] {
            assert_eq!(format!("{variant}"), variant.to_string());
        }
    }

    #[test]
    fn same_variants_are_equal() {
        assert_eq!(TokenType::Access, TokenType::Access);
        assert_eq!(TokenType::Refresh, TokenType::Refresh);
    }

    #[test]
    fn different_variants_are_not_equal() {
        assert_ne!(TokenType::Access, TokenType::Refresh);
    }

    #[test]
    fn clone_produces_equal_value() {
        assert_eq!(TokenType::Access.clone(), TokenType::Access);
        assert_eq!(TokenType::Refresh.clone(), TokenType::Refresh);
    }

    #[test]
    fn debug_contains_variant_name() {
        assert!(format!("{:?}", TokenType::Access).contains("Access"));
        assert!(format!("{:?}", TokenType::Refresh).contains("Refresh"));
    }

    #[test]
    fn display_strings_are_distinct() {
        assert_ne!(TokenType::Access.to_string(), TokenType::Refresh.to_string());
    }
}
