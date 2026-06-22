//! Telemetry Library Module
//!
//! This library provides telemetry configuration types used to set up
//! application logging and tracing.
//!

// TODO: I think this can be done better

//--- Import crate modules

mod domain;
mod error;
mod init;

//--- Re-export for clean imports by other crates

/// Telemetry level configuration.
pub use domain::TracingLevels;

/// Telemetry error type.
pub use error::Error;

/// Telemetry Result type alias used across the telemetry module.
pub type Result<T> = std::result::Result<T, Error>;

/// Initialises the global tracing subscriber for the application.
pub use init::init;
