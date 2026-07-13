//! Command-line argument definitions for the BitNode Console server binaries.
//!
//! [`Cli`] is parsed by [`Settings::parse`] and applied as the highest-priority
//! configuration source, overriding config files and environment variables.

use std::path::PathBuf;

/// Command-line arguments accepted by the BitNode Console server binaries.
///
/// These flags override all other configuration sources (config files,
/// environment variables, built-in defaults).
#[derive(Debug, Default, clap::Parser)]
#[command(about)]
pub struct Cli {
    /// Path to a configuration file (overrides all config paths).
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Set the log level (off | error | warn | info | debug | trace).
    #[arg(short = 'l', long, value_name = "LEVEL")]
    pub log_level: Option<lib_tracing::Levels>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[test]
    fn no_flags_leaves_all_fields_none() {
        let cli = Cli::try_parse_from(["bin"]).unwrap();
        assert!(cli.config.is_none());
        assert!(cli.log_level.is_none());
    }

    #[test]
    fn long_config_flag_sets_path() {
        let cli = Cli::try_parse_from(["bin", "--config", "/etc/bitnode.conf"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/etc/bitnode.conf")));
    }

    #[test]
    fn short_config_flag_sets_path() {
        let cli = Cli::try_parse_from(["bin", "-c", "/etc/bitnode.conf"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/etc/bitnode.conf")));
    }

    #[test]
    fn long_log_level_flag_sets_level() {
        let cli = Cli::try_parse_from(["bin", "--log-level", "debug"]).unwrap();
        assert_eq!(cli.log_level, Some(lib_tracing::Levels::DEBUG));
    }

    #[test]
    fn short_log_level_flag_sets_level() {
        let cli = Cli::try_parse_from(["bin", "-l", "debug"]).unwrap();
        assert_eq!(cli.log_level, Some(lib_tracing::Levels::DEBUG));
    }

    #[test]
    fn all_log_level_values_parse() {
        let cases = [
            ("off", lib_tracing::Levels::OFF),
            ("error", lib_tracing::Levels::ERROR),
            ("warn", lib_tracing::Levels::WARN),
            ("info", lib_tracing::Levels::INFO),
            ("debug", lib_tracing::Levels::DEBUG),
            ("trace", lib_tracing::Levels::TRACE),
        ];
        for (input, expected) in cases {
            let cli = Cli::try_parse_from(["bin", "--log-level", input])
                .unwrap_or_else(|e| panic!("failed to parse level '{input}': {e}"));
            assert_eq!(cli.log_level, Some(expected));
        }
    }

    #[test]
    fn invalid_log_level_returns_error() {
        let result = Cli::try_parse_from(["bin", "--log-level", "verbose"]);
        assert!(result.is_err());
    }

    #[test]
    fn config_and_log_level_can_be_combined() {
        let cli = Cli::try_parse_from([
            "bin",
            "--config",
            "/tmp/bitnode.conf",
            "--log-level",
            "warn",
        ])
        .unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/bitnode.conf")));
        assert_eq!(cli.log_level, Some(lib_tracing::Levels::WARN));
    }
}
