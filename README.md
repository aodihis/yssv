# YSSV — Your Stupid SQL Viewer

A simple native desktop app to browse and query your **MySQL** and **PostgreSQL** databases. No Electron, no browser, no nonsense — just a fast native UI built with Rust and [egui](https://github.com/emilk/egui).

---

## Download

Grab the latest pre-built binary for your platform from the [Releases](https://github.com/aodihis/yssv/releases) page.

| Platform | File |
|----------|------|
| Windows  | `yssv-windows-x86_64.exe` |
| macOS (Intel) | `yssv-macos-x86_64` |
| macOS (Apple Silicon) | `yssv-macos-aarch64` |
| Linux    | `yssv-linux-x86_64` |

No installer needed — just download and run.

> **macOS note:** On first launch you may need to right-click → Open to bypass Gatekeeper.
>
> **Linux note:** Make the binary executable first: `chmod +x yssv-linux-x86_64`

---

## Features

**Connection manager**
- PostgreSQL and MySQL support
- Save, duplicate, and delete connections
- Color labels and group organisation with drag-and-drop reordering
- SSH tunnel support — password, key file, or SSH agent auth
- Test connection button with real round-trip latency

**Data explorer**
- Collapsible sidebar tree — databases → schemas → tables with row counts
- Tabbed interface with right-click tab actions (close, close others, close to left/right)
- **Data tab** — paginated row grid with row selection and copy to clipboard
- **Structure tab** — column definitions with PK / FK / nullable annotations
- Inline data editing with a pending-changes review before commit
- SQL query editor with syntax highlighting and Ctrl+Enter to run

**UI**
- Dark and light themes
- Compact / Regular / Comfy row density
- Structured logs with automatic daily rotation

---

## Settings

App settings (theme, density, sidebar width) are stored alongside the connection database:

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\yssv\` |
| macOS / Linux | `~/.config/yssv/` |

**Connections database:** `connections.db` in the folder above — a plain SQLite file. You can back it up or copy it between machines.

---

## Logs

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\yssv\logs\yssv.log.YYYY-MM-DD` |
| macOS / Linux | `~/.config/yssv/logs/yssv.log.YYYY-MM-DD` |

Log files older than 7 days are deleted automatically on startup. Override with `YSSV_LOG_KEEP_DAYS=30`.

**Log level** (default `info` in release builds, `debug` in dev):

```sh
# Linux / macOS
YSSV_LOG=debug ./yssv

# Windows PowerShell
$env:YSSV_LOG = "debug"
.\yssv.exe
```

---

## Build from source

**Requirements:** Rust 1.80+ via [rustup](https://rustup.rs). No system C libraries needed — TLS uses pure-Rust Rustls.

**Linux only** — install these system packages first:
```sh
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxkbcommon-dev libssl-dev
```

```sh
# Clone
git clone https://github.com/aodihis/yssv.git
cd yssv

# Run in development mode
cargo run

# Build a release binary
cargo build --release
./target/release/yssv          # Linux / macOS
.\target\release\yssv.exe      # Windows
```

---

## Testing

```sh
# Unit tests — no database required, fast
cargo test --lib

# Integration tests (UI smoke tests — no database required)
cargo test --test integration

# All tests including database tests — requires live PostgreSQL and MySQL
export YSSV_TEST_PG_URL=postgres://user:pass@localhost/testdb
export YSSV_TEST_MYSQL_URL=mysql://user:pass@localhost/testdb
cargo test -- --include-ignored
```

---

## License

See [LICENSE](LICENSE).
