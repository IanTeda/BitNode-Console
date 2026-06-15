//! # Telemetry Configuration
//!
//! This module provides configuration structures for the telemetry system in the Personal Ledger application.
//!
//! The telemetry configuration allows users to customize logging verbosity and behavior through
//! configuration files, environment variables, or programmatic settings. It serves as the
//! bridge between user preferences and the telemetry initialization system.

use crate::domain;

/// Default telemetry level if none is provided.
const DEFAULT_TELEMETRY_LEVEL: domain::TelemetryLevels = domain::TelemetryLevels::INFO;

/// Settings struct for telemetry library crate.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct TelemetrySettings {
    pub telemetry_level: domain::TelemetryLevels,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            telemetry_level: DEFAULT_TELEMETRY_LEVEL,
        }
    }
}

impl TelemetrySettings {
    #[must_use]
    pub const fn telemetry_level(&self) -> domain::TelemetryLevels {
        self.telemetry_level
    }
}
