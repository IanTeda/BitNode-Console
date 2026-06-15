//-- ./backend/libs/lib_settings/src/lib.rs

//! Settings Library Module
//!
//! This library is used to load and validate configuration files.

mod error;
mod settings;

/// Settings loading and validation errors.
pub use error::{SettingsError, SettingsResult};

/// Application settings.
pub use settings::Settings;
