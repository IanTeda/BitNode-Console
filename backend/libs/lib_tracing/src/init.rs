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
pub fn init(telemetry_level: Option<crate::Levels>) -> crate::Result<()> {
    let env_filter = build_env_filter(telemetry_level);

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer());

    tracing_log::LogTracer::init()
        .map_err(|err| crate::Error::Generic(format!("Log tracer initialisation failed: {err}")))?;

    set_global_default(registry).map_err(|err| {
        crate::Error::Generic(format!("Failed to set global default subscriber: {err}"))
    })?;

    match std::env::var("RUST_LOG") {
        Ok(rust_log) => tracing::info!(
            "Tracing initialised; RUST_LOG={rust_log:?} overrides configured level ({})",
            default_level_filter(telemetry_level),
        ),
        Err(_) => tracing::info!(
            "Tracing initialised at level: {}",
            default_level_filter(telemetry_level),
        ),
    }

    Ok(())
}

/// Builds the [`EnvFilter`] used by [`init`].
///
/// The `RUST_LOG` environment variable takes precedence when set; otherwise
/// `telemetry_level` is used, falling back to [`DEFAULT_LEVEL_FILTER`] when
/// `telemetry_level` is `None`.
fn build_env_filter(telemetry_level: Option<crate::Levels>) -> EnvFilter {
    let default_directive = default_level_filter(telemetry_level).into();

    EnvFilter::builder().with_default_directive(default_directive).from_env_lossy()
}

/// Resolves the [`LevelFilter`] to use as the default directive when
/// `telemetry_level` is configured, falling back to [`DEFAULT_LEVEL_FILTER`]
/// when `telemetry_level` is `None`.
///
/// This is independent of the `RUST_LOG` environment variable, which is
/// applied separately (and takes precedence) in [`build_env_filter`].
fn default_level_filter(telemetry_level: Option<crate::Levels>) -> LevelFilter {
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
            default_level_filter(Some(crate::Levels::TRACE)),
            LevelFilter::TRACE
        );
    }

    #[test]
    fn default_level_filter_respects_off_level() {
        assert_eq!(
            default_level_filter(Some(crate::Levels::OFF)),
            LevelFilter::OFF
        );
    }

    #[test]
    fn build_env_filter_does_not_panic() {
        let _ = build_env_filter(None);
        let _ = build_env_filter(Some(crate::Levels::TRACE));
    }

    #[test]
    fn init_sets_global_subscriber_once() {
        // The global subscriber and `log` bridge can each only be installed
        // once per process, so the first call succeeds and any subsequent
        // call fails.
        assert!(init(Some(crate::Levels::DEBUG)).is_ok());
        assert!(init(None).is_err());
    }
}
