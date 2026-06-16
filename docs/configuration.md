# Configuration

## Overview

The BitNode Console application uses a hierarchical or layered configuration approach. It grabs configuration files from different directories and applies any settings in the configuration file with the higher rank. The hierarchy uses the following order of importance, with the higher-numbered configuration files and any settings within them  superseding any settings below them.

Here’s how the order works: if a setting appears in more than one file, the configuration file from the higher-ranked location wins. The order of the directory (configuration file) hierarchy is as follows:

1. Default Setting Values (lowest): These values are built-in, hard-coded default values.
2. System Directory: The system-wide config directory, typically `/etc/bitnode-console/bitnode-console.conf`.
3. User Directory: The user-specific config directory, typically `~/.local/share/bitnode-console.conf` or `~/.config/bitnode-console.conf`
4. Executable Directory: The directory where the actual executable `bitnode-console` file physically lives on disk.
5. Working Directory: The directory you are "in" when you run the binary (the folder your shell is currently in when you run `bitnode-console`).
6. Explicit Config File: The config file path passed into the parse method. I.e., the --config --c CLI argument is used after the bitnode-console binary.
7. Environment Variable (Heights): The environment variables with the prefix BITNODE_CONSOLE_, which are read from the process environment when the binary is run.

## Settings

Settings are grouped into areas of concern and are fully described within the following sections:

* Application: Overarching settings that affect the application as a whole
* Tracing (Log): Settings that control how the application uses and manages tracing (logs)
* Web: Settings that control how the application serves the ReactJS frontend web application

### Config file format

Config files use INI format with one section per settings group:

```ini
[application]
setting = false

[tracing]
enabled = true
level = info
show_settings_startup = false

[web]
host = 127.0.0.1
port = 8090
```

All sections and their keys are optional; any omitted value falls back
to the built-in default for that field.

### Settings reference

#### `[application]`

Controls application-level behaviour.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `setting` | `bool` | `false` | Log the active settings to the tracing output at startup |

#### `[tracing]`

Controls the tracing / logging subsystem.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable or disable tracing output entirely |
| `level` | `string` | `info` | Minimum log level: `off`, `error`, `warn`, `info`, `debug`, `trace` |
| `show_settings_startup` | `bool` | `false` | Print the active settings via `tracing::info!` at startup |

#### `[web]`

Controls the HTTP server.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | `string` | `127.0.0.1` | Interface to bind the HTTP listener to |
| `port` | `u16` | `8090` | Port to bind the HTTP listener to |

### Environment variables

Any setting can be overridden with an environment variable following the
pattern `BITNODE_CONSOLE_<SECTION>_<KEY>` (all uppercase):

```sh
# Override web port
BITNODE_CONSOLE_WEB_PORT=9100

# Override tracing level
BITNODE_CONSOLE_TRACING_LEVEL=debug

# Disable tracing
BITNODE_CONSOLE_TRACING_ENABLED=false
```

Environment variables take precedence over all config files.
