# YSSV — Agent Instructions

## Project Overview
YSSV is a native desktop SQL viewer built in Rust with egui/eframe.
UI follows atomic design: atoms → molecules → pages, all under `src/ui/`.

---

## UI Component Rules

Use existing atoms and molecules — never raw egui widgets when an atom already covers the case.
Atoms live in `src/ui/atoms/`, molecules in `src/ui/molecules/`. Check there first.

- If a needed component doesn't exist, create it as an atom/molecule, then use it.
- Never inline `egui::TextEdit`, `egui::Button`, etc. in page code when an atom covers the case.
- Never inline modal/overlay patterns in page code — use the `alert_dialog` molecule or create a dedicated one.

---

## Theme & Colors

Always use `ThemeColors::from_ui(ui)` — never hardcode `Color32` in page code.
Named color constants live in `crate::theme::colors`.

---

## Async & Events

- All async DB work goes through `mpsc::SyncSender<AppEvent>` — never block the UI thread.
- Spawn with `app.tokio.spawn(async move { ... tx.send(AppEvent::...) ... })`.
- Call `ctx.request_repaint()` after sending an event.
- `apply_event` in `app.rs` handles all `AppEvent` variants — add new variants there.

---

## State

- `app.conn_page` — `ConnectionsPageState`: form, test status, selected id, is_new flag.
- `app.explorer` — `Option<ExplorerState>`: databases, tabs, filter, active db.
- `app.settings` — theme, density, sidebar width.
- Never store derived state that can be computed from the above.

---

## Code Style

Function ordering within any module or `impl` block:
1. `pub` / `pub(crate)` before private, alphabetical within each group.
2. `new` is always first in an `impl` block.

- No comments unless the WHY is non-obvious.
- No docstrings.
- Prefer `match` producing a value over chains of `if` blocks.
- Avoid `.clone()` on large types — pass references or use `Arc`.
- Use `Frame::side_top_panel(ui.style()).inner_margin(Margin::same(0))` to zero panel margins. `Frame::new()` breaks background fill.

---

## Logging

Uses the `tracing` crate. Missing logs are a bug — they make production issues impossible to diagnose.

| Level | Use for |
|---|---|
| `error!` | Unrecoverable failures |
| `warn!` | Recoverable errors, unexpected-but-handled states |
| `info!` | User-visible transitions (connected, rows loaded, saved) |
| `debug!` | Internal state changes, async op entry, counts |

Every async op logs at entry and on both success and failure. Every `apply_event` arm logs key fields. Every early-return guard logs a `warn!` explaining why.

Always include context fields: `conn_id`, `db`, `schema`, `table`, `count`, `error`.

---

## Commands

See `Makefile` (run `make help` for the full list). Key targets:

```
make run                  # run in dev mode
make test-lib             # unit tests (no DB needed)
make test-integration     # integration suite
make test-e2e             # e2e suite (requires Docker)
make lint                 # clippy -D warnings
make fmt                  # auto-format
```

Unit tests go in-module with `#[cfg(test)]`. Test public functions; don't test egui rendering.
