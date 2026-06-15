# YSSV — SQL Data Viewer

A native desktop SQL client built with Rust and [egui](https://github.com/emilk/egui). Connects to PostgreSQL and MySQL databases. Inspired by SequelPro and DataGrip.

## Features

**Connection manager**
- PostgreSQL and MySQL support
- Save, duplicate, and delete connections (delete requires confirmation)
- Color labels and smart group autocomplete — pick an existing group or type a new one
- SSH tunnel configuration (password, key file, or agent auth)
- Test connection with real round-trip latency display

**Data explorer**
- Browse databases, schemas, and tables in a collapsible sidebar tree
- Data tab — paginated row grid with row selection, cell highlight, and copy to clipboard
- Structure tab — column definitions with PK / FK / nullable annotations

**UI**
- Dark and light themes with a single toggle
- Compact / Regular / Comfy row density
- Structured logs with automatic daily rotation

## Requirements

- Rust 1.75 or later (`rustup` is the recommended installer)
- No system libraries required — TLS uses Rustls (pure Rust), no `libpq` or MySQL C connector needed

## Building

```sh
# Debug build (fastest, includes debug logging to stderr)
make build
# or
cargo build

# Optimised release build
make release
# or
cargo build --release
```

## Running

```sh
# Development — debug log level, output to stderr + log file
make run

# Release binary
make run-release

# Override log level (works on all platforms via make)
make run-log LEVEL=trace
make run-log LEVEL=yssv=debug,sqlx=warn
```

On Windows without `make`, use PowerShell:

```powershell
cargo run                          # dev build

$env:YSSV_LOG = "trace"
cargo run                          # dev build with custom level

cargo build --release
.\target\release\yssv.exe          # release build
```

## Testing

```sh
# All tests including #[ignore] — requires live DB servers and Docker
export YSSV_TEST_PG_URL=postgres://user:pass@localhost/testdb
export YSSV_TEST_MYSQL_URL=mysql://user:pass@localhost/testdb
make test-all
# or
cargo test -- --include-ignored

# Unit tests only — fastest, no database required
make test-lib
# or
cargo test --lib

# Integration suite only (tests/integration/)
make test-integration
# or
cargo test --test integration

# E2E suite only (tests/e2e/) — requires Docker
make test-e2e
# or
cargo test --test e2e --features e2e
```

On Windows (PowerShell):

```powershell
# All tests (needs live DB + Docker)
$env:YSSV_TEST_PG_URL = "postgres://user:pass@localhost/testdb"
$env:YSSV_TEST_MYSQL_URL = "mysql://user:pass@localhost/testdb"
make test-all
# or
cargo test -- --include-ignored
```

## Code quality

```sh
make check       # type-check
make lint        # clippy -D warnings
make fmt         # auto-format
make fmt-check   # check formatting in CI
```

## Logging

### Log levels

Levels in increasing severity — setting a level includes everything above it.

| Level   | When used |
|---------|-----------|
| `trace` | Very verbose internal detail — library internals |
| `debug` | Developer-facing events (connections, queries, page loads) |
| `info`  | Normal operational events (startup, connected, loaded) |
| `warn`  | Recoverable problems (connection fallback, auth fail) |
| `error` | Unrecoverable errors |

**Default**: `debug` in development builds, `info` in release builds.

### Environment variables

| Variable              | Default | Description |
|-----------------------|---------|-------------|
| `YSSV_LOG`            | `debug` / `info` | Log filter — level and/or per-module directives |
| `YSSV_LOG_KEEP_DAYS`  | `7`     | Days of rolling log files to retain before deletion |
| `YSSV_RENDERER`       | `wgpu`  | GPU renderer: `wgpu` (default) or `glow` (OpenGL, better compatibility) |

### Log filter syntax

```sh
# Single level — applies to everything
YSSV_LOG=trace

# Per-crate — quiet sqlx, verbose app
YSSV_LOG=yssv=debug,sqlx=warn

# Single module at trace
YSSV_LOG=yssv::core::drivers=trace,yssv=info

# Everything off except errors
YSSV_LOG=error
```

### Log destinations

- **Stderr** — human-readable, colored. Always on in debug builds; in release builds only when `YSSV_LOG` is set.
- **Rolling file** — compact format, no ANSI codes, always active.

### Log file locations

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\yssv\logs\yssv.log.YYYY-MM-DD` |
| macOS    | `~/.config/yssv/logs/yssv.log.YYYY-MM-DD` |
| Linux    | `~/.config/yssv/logs/yssv.log.YYYY-MM-DD` |

Files older than `YSSV_LOG_KEEP_DAYS` days are deleted automatically on startup.

### Examples

**Linux / macOS:**

```sh
# Release binary — default INFO to file only
./yssv

# Enable DEBUG to stderr + file
YSSV_LOG=debug ./yssv

# Keep 30 days of logs
YSSV_LOG_KEEP_DAYS=30 ./yssv
```

**Windows (PowerShell):**

```powershell
# Set env vars before running — inline assignment doesn't work in PowerShell
$env:YSSV_LOG = "debug"
.\target\release\yssv.exe

# Multiple vars
$env:YSSV_LOG = "yssv=debug,sqlx=warn"
$env:YSSV_LOG_KEEP_DAYS = "14"
.\target\release\yssv.exe
```

**Windows (cmd.exe):**

```bat
set YSSV_LOG=debug && yssv.exe
```

**Via Makefile (all platforms):**

```sh
make run-log LEVEL=debug
make run-log LEVEL=yssv=debug,sqlx=warn
```

## Data storage

Connection metadata is stored in a local SQLite database:

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\yssv\connections.db` |
| macOS / Linux | `~/.config/yssv/connections.db` |


## License

See [LICENSE](LICENSE).
