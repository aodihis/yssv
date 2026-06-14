# YSSV — Claude Instructions

## Project Overview
YSSV is a native desktop SQL viewer built in Rust with `egui 0.31` / `eframe 0.31`.
Atomic-design UI system: atoms → molecules → pages. See PLAN.md for full architecture.

---

## UI Component Rules

### Always use existing atoms and molecules — never raw egui widgets for UI that has an atom

| Need | Use | Import |
|---|---|---|
| Text field | `text_input(ui, value, placeholder, icon)` | `crate::ui::atoms::input::text_input` |
| Password field | `password_input(ui, value, placeholder, icon)` | `crate::ui::atoms::input::password_input` |
| Monospace field | `mono_input(ui, value, placeholder, icon)` | `crate::ui::atoms::input::mono_input` |
| File picker field | `file_input(ui, value, placeholder)` | `crate::ui::atoms::input::file_input` |
| Primary CTA button | `primary_button(ui, label)` | `crate::ui::atoms::button::primary_button` |
| Compact inline button | `compact_button(ui, label)` | `crate::ui::atoms::button::compact_button` |
| Dropdown/select | `dropdown(ui, id, value, options)` | `crate::ui::atoms::dropdown::dropdown` |
| Toggle switch | `light_switch(ui, value)` | `crate::ui::atoms::light_switch::light_switch` |
| Color dot label | `label_dot(ui, color)` | `crate::ui::atoms::label_dot` |
| Badge/pill text | `badge(ui, text)` | `crate::ui::atoms::badge` |
| SVG icon | `svg_icon(ui, Icon::X, size, color)` | `crate::ui::atoms::icon::svg_icon` |
| Icon image (for buttons) | `icon_image(Icon::X, size, color)` | `crate::ui::atoms::icon::icon_image` |
| Horizontal divider | see `crate::ui::atoms::divider` | — |
| Connection list item | `conn_item(...)` | `crate::ui::molecules::conn_item` |
| Sidebar tree row | `tree_row(ui, TreeRowConfig { ... })` | `crate::ui::molecules::tree_row` |
| Data grid cell | `render_cell(ui, col, value)` | `crate::ui::molecules::data_cell` |
| Data results table | `data_table(ui, columns, rows, row_height, selected) -> Option<usize>` | `crate::ui::molecules::data_table` |
| Pagination status bar | `status_bar(ui, tab)` | `crate::ui::molecules::status_bar` |
| Tab bar | `tab_bar(ui, tabs, active)` | `crate::ui::molecules::tab_bar` |
| Error/alert dialog | `error_dialog(ui, id, title, message) -> bool` | `crate::ui::molecules::alert_dialog` |

**If an atom/molecule doesn't exist yet for a new need, create it in `src/ui/atoms/` or `src/ui/molecules/` first, then use it.**
Never inline raw `egui::TextEdit`, `egui::Button`, etc. directly in page code when an atom covers the case.
Never inline modal/overlay patterns in page code — use `error_dialog` for error popups, or create a dedicated molecule if the pattern differs.

---

## Theme & Colors

Always use `ThemeColors::from_ui(ui)` for all colors — never hardcode `Color32` values in page code.

```rust
let tc = ThemeColors::from_ui(ui);
// tc.text_primary, tc.text_secondary, tc.text_disabled
// tc.background, tc.surface, tc.surface_secondary
// tc.border, tc.border_muted, tc.field_border
// tc.button_primary_bg, tc.button_secondary_bg
// tc.success, tc.error
// tc.button_primary_bg (accent)
```

Named color constants (connection labels, icons) live in `crate::theme::colors`.

---

## Async & Events

- All async DB work goes through `mpsc::SyncSender<AppEvent>` — never block the UI thread.
- Async tasks: `app.tokio.spawn(async move { ... tx.send(AppEvent::...) ... })`.
- After sending an event call `ctx.request_repaint()` to wake the frame loop.
- `apply_event` in `app.rs` handles all `AppEvent` variants — add new variants there.

---

## State

- `app.conn_page` — `ConnectionsPageState`: form, test status, selected id, is_new flag.
- `app.explorer` — `Option<ExplorerState>`: databases, tabs, filter, active db.
- `app.settings` — theme, density, sidebar width.
- `TestStatus`: `Idle | Testing | Ok | Failed(String)` — drives connection detail UI.
- Never store derived state that can be computed from the above.

---

## Code Style

### Function ordering

Within any module or `impl` block, always order functions as follows:

1. **Public before private** — all `pub` / `pub(crate)` functions come before private (`fn`) functions.
2. **Alphabetical within each visibility group.**
3. **`new` is always first** inside an `impl` block (before all other methods, regardless of alphabetical order).

```
// free functions in a module
pub fn alpha(...)   // public, alphabetical
pub fn beta(...)
fn internal_a(...)  // private, alphabetical
fn internal_b(...)

// impl block
impl Foo {
    pub fn new(...) -> Self { ... }   // always first
    pub fn alpha(...) { ... }          // public, alphabetical
    pub fn beta(...) { ... }
    fn helper_a(...) { ... }           // private, alphabetical
    fn helper_b(...) { ... }
}
```

Trait `impl` blocks follow the same alphabetical rule (all methods are effectively public; no `new` convention applies since the trait defines the interface).

- No comments unless the WHY is non-obvious (hidden constraint, workaround, subtle invariant).
- No docstrings. No `// Added for X flow` comments.
- Match arms: prefer data-driven `match` to produce a value over multiple `if` blocks.
- Avoid `.clone()` on large types (QueryResult rows, tab state). Pass references; use `Arc` if shared ownership is needed.
- `Frame::side_top_panel(ui.style()).inner_margin(Margin::same(0))` — use this pattern to zero panel margins while preserving theme styling. `Frame::new()` breaks background fill/stroke.

---

## Logging

The project uses `tracing` crate. Every non-trivial operation MUST have log coverage. Missing logs are a bug, not a style choice — they make production issues impossible to diagnose.

### Level rules

| Level | Use for |
|---|---|
| `tracing::error!` | Unrecoverable failures — things that should never happen |
| `tracing::warn!` | Recoverable errors, unexpected-but-handled states (e.g. no active connection, event for unknown id) |
| `tracing::info!` | Important user-visible transitions: connected, schemas loaded, rows loaded, connection saved/deleted |
| `tracing::debug!` | Internal state changes, function entry for async ops, counts, decisions |

### Required log points

**Every async operation** must log at entry (`debug!`) and on both success (`info!` or `debug!`) and failure (`warn!` or `error!`):
```rust
tracing::debug!(db = %db, "load_schemas: requested");
// ... spawn ...
tracing::debug!(db = %db, "load_schemas: starting async fetch");
match result {
    Ok(x)  => tracing::info!(db = %db, count = x.len(), "load_schemas: success"),
    Err(e) => tracing::warn!(db = %db, error = %e.message, "load_schemas: failed"),
}
```

**Every `apply_event` arm** must log the key fields of the event.

**Every guard / early-return** for missing state must log a `warn!` explaining why:
```rust
} else {
    tracing::warn!(db = %db, "SchemasLoaded: no matching database in explorer");
}
```

### Field naming conventions

Always include relevant context fields — don't log bare messages:
- `conn_id = %id` on all connection-related events
- `db = %db`, `schema = %schema`, `table = %table` on DB operations
- `count = n` or `row_count = n` on result sets
- `error = %e.message` on failures

---

## Testing

```
cargo test --lib          # unit tests
cargo test --test storage_tests   # SQLite integration
cargo check               # fast compile check before running
```

Write unit tests in-module with `#[cfg(test)]`. Test public functions; don't test egui rendering.

---

## Remaining Phase 2 Work

- [x] Table row counts in sidebar (PG: `reltuples` via `pg_class` LEFT JOIN; MySQL: `TABLE_ROWS`)
- [ ] Connection status indicator (latency ping in explorer footer)

## Phase 3 Next

- Delete connection with confirmation dialog
- Duplicate connection
- Test Connection shows real latency ms
