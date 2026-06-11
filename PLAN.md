# YSSV — Development Plan

YSSV is a native desktop SQL data viewer built in Rust using `egui`/`eframe`.
Inspired by SequelPro and DataGrip, designed from scratch with a clean minimal aesthetic.

---

## Project Structure

```
src/
├── core/               # Domain models, drivers, storage
│   ├── connections/    # Connection model + SQLite CRUD
│   ├── drivers/        # Async DB driver trait + PostgreSQL + MySQL
│   ├── schema/         # DbInfo, SchemaInfo, TableInfo
│   ├── results/        # QueryResult, ColumnDef
│   └── ssh/            # SSH tunnel config model
├── pages/              # Screen-level state machines
│   ├── connections/    # Connection manager state + UI
│   ├── explorer/       # Data explorer state + UI (tree, tabs, grid)
│   └── settings/       # App settings (theme, density, sidebar)
├── ui/                 # Atomic-design UI components
│   ├── atoms/          # button, badge, input, toggle, label_dot, divider
│   ├── molecules/      # conn_item, tree_row, data_cell, tab_bar, status_bar
│   └── layouts/        # sidebar_layout, two_pane
├── app.rs              # YssvApp: event loop, screen routing, theme toggle
├── theme.rs            # Color tokens + egui Visuals builder (dark/light)
└── main.rs             # Entry point: tokio runtime + eframe launch
```

---

## Tech Stack

| Layer | Crate | Notes |
|---|---|---|
| GUI | `eframe 0.31` + `egui 0.31` | Native desktop, immediate-mode |
| Data grid | `egui_extras 0.31` | `TableBuilder` for rows |
| Async | `tokio 1` (full) | Runtime stored in app; mpsc channels for results |
| Connection storage | `rusqlite 0.31` (bundled) | SQLite at `%APPDATA%\yssv\connections.db` |
| DB drivers | `sqlx 0.8` (postgres + mysql) | Pure-Rust TLS — no system `libpq`/`libssl` needed |
| Serialization | `serde 1` + `serde_json` | All models derive Serialize/Deserialize |
| IDs | `uuid 1` (v4) | Connection IDs |

---

## Phases

Each phase ships with **unit tests** (`#[cfg(test)]` in-module) and **integration tests** (`tests/` directory).

### Phase 1 — Foundation ✅
**Status: Complete** (`commit 5666267`)

- [x] Project skeleton: Cargo.toml, directory structure
- [x] Theme system: dark/light color tokens matching design spec
- [x] Core models: Connection, SshConfig, DbInfo, QueryResult
- [x] SQLite storage: CRUD for connections + groups
- [x] UI components: atoms, molecules, layouts (atomic design)
- [x] Connection Manager page: grouped list + form + SSH section
- [x] Database Explorer page: sidebar tree + table tabs + data grid
- [x] PostgreSQL driver: connect, list databases/schemas/tables, fetch rows
- [x] MySQL driver: same interface
- [x] Error handling: DbError with install-hint for missing server
- [x] App event loop: tokio async + mpsc channels + repaint
- [x] Theme toggle in titlebar (dark ↔ light)
- [x] 40 unit tests + 6 integration tests, all passing

---

### Phase 2 — Live Connections & Data Browsing
**Goal**: Real end-to-end flow from "Connect" button to viewing live table data.

- [x] Wire "Connect" → store live `ActiveConnection` in app state
- [x] Sidebar tree: load schemas lazily on database node expand
- [x] Table row counts: show real counts in sidebar (use `reltuples` estimate for PG)
- [x] Structure view: render `describe_table` output in the data grid
- [x] Column type annotations in grid headers (PK / FK markers)
- [x] Row selection and highlight
- [x] Copy cell value to clipboard
- [ ] Connection status indicator (latency ping)

---

### Phase 3 — Connection UX Polish
**Goal**: Full connection management workflow.

- [x] Delete connection (with confirmation)
- [x] Duplicate connection
- [ ] Drag-and-drop reorder within/across groups
- [x] Test Connection button shows real latency
- [ ] Group rename

---

### Phase 4 — Query Editor
**Goal**: First-class SQL console tab.

- [ ] New "Query" tab type (alongside table tabs)
- [ ] Syntax-highlighted SQL editor (using `egui_code_editor` or similar)
- [ ] Run query → display results in grid below editor
- [ ] Multi-statement support
- [ ] Error display with line/column highlight
- [ ] Keyboard shortcut: Ctrl+Enter to run

---

### Phase 5 — SSH Tunneling
**Goal**: Connect through SSH bastion hosts.

- [ ] SSH tunnel execution using `russh` or `ssh2` crate
- [ ] Forward local port → remote DB port through tunnel
- [ ] Key-file auth (PEM / OpenSSH format)
- [ ] Tunnel status indicator in sidebar footer
- [ ] Auto-reconnect on tunnel drop

---

### Phase 6 — Data Editing (Future)
**Goal**: Edit and insert rows directly in the grid.

- [ ] Inline cell editing (click to edit)
- [ ] Insert new row
- [ ] Delete row (with confirmation)
- [ ] Commit / rollback changes
- [ ] Show pending changes diff before committing

---

## Testing Strategy

| Test type | Location | Run command |
|---|---|---|
| Unit tests | `src/**/*.rs` `#[cfg(test)]` | `cargo test --lib` |
| Integration (SQLite) | `tests/storage_tests.rs` | `cargo test --test storage_tests` |
| Integration (PostgreSQL) | `tests/driver_tests.rs` | `YSSV_TEST_PG_URL=... cargo test --test driver_tests -- --include-ignored` |
| Integration (MySQL) | `tests/driver_tests.rs` | `YSSV_TEST_MYSQL_URL=... cargo test --test driver_tests -- --include-ignored` |

---

## Design Reference

The UI design was created in Claude Design and matches the following spec:

**Dark theme** (default): bg `#11151c`, panel `#0c1015`, accent `#5181ff`, text `#e7ebf2`  
**Light theme**: bg `#ffffff`, panel `#f5f6f8`, accent `#2f5ce6`, text `#1b1f24`  
**Connection labels**: red `#e5484d` · amber `#e0962a` · green `#2ea36b` · blue `#3a83f0` · purple `#8a5cf0` · gray `#8b93a0`  
**Typography**: IBM Plex Sans (UI) + IBM Plex Mono (data/SQL)  
**Density**: Compact (26px rows) · Regular (30px) · Comfy (38px)
