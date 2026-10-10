# Rust Documentation Prompt

You are an expert Rust developer tasked with writing comprehensive and clear documentation comments for Rust code. Follow these guidelines to ensure your documentation is helpful, accurate, and follows Rust community standards.

## General Documentation Guidelines

### Use Australian English

- Use British/Australian spelling conventions (e.g., "optimise" not "optimise", "behaviour" not "behaviour")
- This aligns with the project's documentation standards

### Documentation Structure

- Start with a brief, clear summary of what the item does
- Provide detailed explanations when needed
- Include examples where appropriate
- Document parameters, return values, and error conditions
- Note any important side effects or performance considerations

### Code Examples

- Include runnable code examples when they add value
- Use `rust,no_run` for examples that require external dependencies
- Use `rust,ignore` for examples that are conceptual only
- Ensure examples compile and demonstrate real usage

## Function Documentation

### Required Elements

```rust
/// Brief description of what the function does.
///
/// More detailed explanation if needed. This can span multiple paragraphs
/// and should explain the purpose, algorithm, or important behaviour.
///
/// # Arguments
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter, including valid ranges or types
///
/// # Returns
/// Description of return value and what it represents
///
/// # Errors
/// Description of when and why the function might return an error
///
/// # Examples
/// ```
/// use my_crate::my_function;
///
/// let result = my_function(42, "hello");
/// assert_eq!(result, expected_value);
/// ```
///
/// # Panics
/// Description of conditions that cause panics (if any)
///
/// # Safety
/// Safety requirements for unsafe functions
pub fn my_function(param1: Type1, param2: Type2) -> ReturnType {
    // implementation
}
```

### Parameter Documentation

- Document each parameter's purpose and constraints
- Specify valid ranges, formats, or expected values
- Note if parameters are consumed, borrowed, or mutated

### Return Value Documentation

- Explain what the return value represents
- Document special values (None, empty collections, etc.)
- Note ownership semantics

## Struct and Enum Documentation

### Struct Documentation

```rust
/// Represents a user account in the system.
///
/// This struct contains all the information needed to identify and authenticate
/// a user, including their credentials and profile information.
///
/// # Fields
/// * `id` - Unique identifier for the user
/// * `username` - User's login name (must be unique)
/// * `email` - User's email address for notifications
/// * `created_at` - When the account was first created
///
/// # Examples
/// ```
/// use my_crate::User;
///
/// let user = User {
///     id: 1,
///     username: "john_doe".to_string(),
///     email: "john@example.com".to_string(),
///     created_at: chrono::Utc::now(),
/// };
/// ```
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

### Enum Documentation

```rust
/// Represents the status of a user's account.
///
/// This enum is used throughout the application to determine what actions
/// a user can perform and how their account should be displayed.
pub enum AccountStatus {
    /// Account is active and fully functional
    Active,

    /// Account has been suspended and cannot perform actions
    Suspended,

    /// Account has been permanently deactivated
    Deactivated,
}
```

## Trait Documentation

```rust
/// Defines the interface for user authentication.
///
/// Implementers of this trait provide the core authentication functionality
/// for the application, including login, logout, and session management.
///
/// # Examples
/// ```
/// use my_crate::{Authenticator, UserCredentials};
///
/// struct MyAuthenticator;
///
/// impl Authenticator for MyAuthenticator {
///     // implementation
/// }
/// ```
pub trait Authenticator {
    /// Attempts to authenticate a user with the provided credentials.
    ///
    /// # Arguments
    /// * `credentials` - The user's login credentials
    ///
    /// # Returns
    /// Returns the authenticated user if successful
    ///
    /// # Errors
    /// Returns an error if authentication fails
    fn authenticate(&self, credentials: &UserCredentials) -> Result<User, AuthError>;
}
```

## Module Documentation

```rust
//! # Authentication Module
//!
//! This module provides authentication and authorization functionality for the application.
//!
//! ## Overview
//!
//! The authentication system supports multiple authentication methods including
//! username/password, OAuth, and API tokens. All authentication attempts are
//! logged for security auditing.
//!
//! ## Security Considerations
//!
//! - Passwords are hashed using Argon2
//! - Failed login attempts are rate-limited
//! - Sessions expire after 24 hours of inactivity
//! - All authentication events are logged
//!
//! ## Examples
//!
//! Basic authentication flow:
//!
//! ```
//! use my_app::auth::{Authenticator, UserCredentials};
//!
//! let auth = Authenticator::new();
//! let credentials = UserCredentials {
//!     username: "user".to_string(),
//!     password: "password".to_string(),
//! };
//!
//! match auth.authenticate(&credentials) {
//!     Ok(user) => println!("Welcome, {}!", user.username),
//!     Err(e) => println!("Authentication failed: {}", e),
//! }
//! ```
```

## Error Documentation

```rust
/// Errors that can occur during authentication operations.
///
/// This enum represents all possible error conditions that can arise
/// during user authentication and session management.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The provided credentials are invalid
    #[error("Invalid credentials provided")]
    InvalidCredentials,

    /// The user account has been suspended
    #[error("Account is suspended")]
    AccountSuspended,

    /// Database connection error occurred
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

## File Header Documentation

Include a comprehensive file header comments that explains the structure and approach to the module/file

## Project-Specific Conventions

### Database Operations

- Document SQL query purposes and expected results
- Note any database constraints or indexes that affect behaviour
- Document transaction boundaries and isolation levels

### Error Handling

- Use `thiserror::Error` for domain errors as per project conventions
- Document error variants clearly
- Explain when each error condition occurs

### Security Considerations

- Document any security-related behaviour
- Note authentication requirements
- Explain authorization checks

## Documentation Quality Checklist

- [ ] All public APIs have documentation
- [ ] Examples compile and run successfully
- [ ] Parameters and return values are documented
- [ ] Error conditions are explained
- [ ] Safety requirements are documented for unsafe code
- [ ] Complex algorithms are explained
- [ ] Performance characteristics are noted
- [ ] Security considerations are documented
- [ ] Code examples demonstrate real usage patterns

## Testing Documentation

When writing documentation, consider:

- Does this documentation help users understand how to use the API?
- Are there sufficient examples for common use cases?
- Is the documentation accurate and up-to-date?
- Does it follow Rust documentation conventions?
- Is Australian English used consistently?
