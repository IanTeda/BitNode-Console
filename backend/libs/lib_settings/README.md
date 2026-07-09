---
title: lib_settings
description: Layered configuration loader for the BitNode Console backend.
crate: lib_settings
---

## Overview

`lib_settings` loads and merges application configuration from multiple
sources into a single typed [`Settings`] struct. Sources are applied in
precedence order — later sources override earlier ones — so a value set
in an environment variable will always win over the same key in a config
file.

## Configuration sources

Sources are applied from lowest to highest precedence:

| # | Source | Example location |
|---|--------|-----------------|
| 1 | Built-in defaults | Hard-coded in source |
| 2 | System config file | `/etc/bitnode_console/bitnode_console.conf` |
| 3 | User config file | `~/.config/bitnode_console/bitnode_console.conf` |
| 4 | Executable directory | `<binary_dir>/bitnode_console.conf` |
| 5 | Working directory | `./bitnode_console.conf` |
| 6 | Explicit config file | Path passed to `Settings::parse(Some(path))` |
| 7 | Environment variables | `BITNODE_CONSOLE_WEB_PORT=9100` |

A source is silently skipped if the file does not exist. An error is
returned if a file exists but cannot be parsed.

## Usage

### Load defaults

```rust
use lib_settings::Settings;

fn main() -> Result<(), lib_settings::SettingsError> {
    // Load from all standard locations; fall back to built-in defaults.
    let settings = Settings::parse(None)?;

    println!("Listening on {}:{}", settings.web.host, settings.web.port);
    Ok(())
}
```

### Load with an explicit config file

```rust
use lib_settings::Settings;
use std::path::Path;

fn main() -> Result<(), lib_settings::SettingsError> {
    let config_path = Path::new("/etc/myapp/custom.conf");
    let settings = Settings::parse(Some(config_path))?;

    println!("Tracing level: {}", settings.tracing.level);
    Ok(())
}
```

### Use settings in the application

```rust
use lib_settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::parse(None)?;

    // Initialise tracing if enabled.
    if settings.tracing.enabled {
        lib_tracing::init(Some(settings.tracing.level))?;

        if settings.tracing.show_settings_startup {
            tracing::info!("Active settings: {settings:#?}");
        }
    }

    // Start the web server.
    let server = lib_web::HttpServer::new(&settings.web.host, settings.web.port);
    server.run().await?;

    Ok(())
}
```

## Module structure

```
lib_settings/
└── src/
    ├── lib.rs              — Public re-exports
    ├── settings.rs         — Settings struct and parse() logic
    ├── application.rs      — ApplicationSettings ([application] section)
    ├── tracing.rs          — TracingSettings     ([tracing] section)
    ├── web.rs              — WebSettings         ([web] section)
    └── error.rs            — SettingsError and SettingsResult
```

## Error handling

[`Settings::parse`] returns a [`SettingsResult<Settings>`], which is an
alias for `Result<Settings, SettingsError>`. Errors include:

- `SettingsError::Parsing` — a config source contained an invalid value
- `SettingsError::Io` — a config file could not be read
- `SettingsError::Generic` — an unexpected error from the `config` crate
