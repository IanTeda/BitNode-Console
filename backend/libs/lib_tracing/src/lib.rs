//! Telemetry Library Module
//!
//! This library provides telemetry configuration types used to set up
//! application logging and tracing.
//!

// TODO: I think this can be done better

//--- Import crate modules

mod domain;
mod error;

//--- Re-export for clean imports by other crates

/// Telemetry level configuration.
pub use domain::TracingLevels;

/// Telemetry error type.
pub type TracingError = error::TracingError;

/// Telemetry Result type alias used across the telemetry module.
pub type TelemetryResult<T> = std::result::Result<T, TracingError>;

use tracing::level_filters::LevelFilter;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{EnvFilter, prelude::*};

//--- Default values

/// Default log level used when no `telemetry_level` is configured and the
/// `RUST_LOG` environment variable is not set.
const DEFAULT_LEVEL_FILTER: LevelFilter = LevelFilter::INFO;

/// Initialises the global tracing subscriber for the application.
///
/// `telemetry_level` sets the default verbosity used when the `RUST_LOG`
/// environment variable is not set (or cannot be parsed). The `RUST_LOG`
/// environment variable always takes precedence when present, allowing the
/// log level to be overridden at runtime without changing configuration.
///
/// This also bridges the `log` crate to `tracing`, so libraries that emit
/// `log` records are captured by the same subscriber.
///
/// # Errors
///
/// Returns [`TelemetryError::Generic`] if a global `log` bridge or `tracing`
/// subscriber has already been installed for this process.
pub fn init(telemetry_level: Option<TracingLevels>) -> TelemetryResult<()> {
    let env_filter = build_env_filter(telemetry_level);

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer());

    tracing_log::LogTracer::init()
        .map_err(|err| TracingError::Generic(format!("Log tracer initialisation failed: {err}")))?;

    set_global_default(registry).map_err(|err| {
        TracingError::Generic(format!("Failed to set global default subscriber: {err}"))
    })?;

    Ok(())
}

/// Builds the [`EnvFilter`] used by [`init`].
///
/// The `RUST_LOG` environment variable takes precedence when set; otherwise
/// `telemetry_level` is used, falling back to [`DEFAULT_LEVEL_FILTER`] when
/// `telemetry_level` is `None`.
fn build_env_filter(telemetry_level: Option<TracingLevels>) -> EnvFilter {
    let default_directive = default_level_filter(telemetry_level).into();

    EnvFilter::builder().with_default_directive(default_directive).from_env_lossy()
}

/// Resolves the [`LevelFilter`] to use as the default directive when
/// `telemetry_level` is configured, falling back to [`DEFAULT_LEVEL_FILTER`]
/// when `telemetry_level` is `None`.
///
/// This is independent of the `RUST_LOG` environment variable, which is
/// applied separately (and takes precedence) in [`build_env_filter`].
fn default_level_filter(telemetry_level: Option<TracingLevels>) -> LevelFilter {
    telemetry_level.map_or(DEFAULT_LEVEL_FILTER, LevelFilter::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_filter_with_none_uses_default() {
        assert_eq!(default_level_filter(None), DEFAULT_LEVEL_FILTER);
    }

    #[test]
    fn default_level_filter_with_some_uses_configured_level() {
        assert_eq!(
            default_level_filter(Some(TracingLevels::TRACE)),
            LevelFilter::TRACE
        );
    }

    #[test]
    fn default_level_filter_respects_off_level() {
        assert_eq!(
            default_level_filter(Some(TracingLevels::OFF)),
            LevelFilter::OFF
        );
    }

    #[test]
    fn build_env_filter_does_not_panic() {
        let _ = build_env_filter(None);
        let _ = build_env_filter(Some(TracingLevels::TRACE));
    }

    #[test]
    fn init_sets_global_subscriber_once() {
        // The global subscriber and `log` bridge can each only be installed
        // once per process, so the first call succeeds and any subsequent
        // call fails.
        assert!(init(Some(TracingLevels::DEBUG)).is_ok());
        assert!(init(None).is_err());
    }
}
