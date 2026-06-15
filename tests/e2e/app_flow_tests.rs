//! End-to-end coverage for `YssvApp`'s async database orchestration.
//!
//! Unlike `render_tests` (which never touches a real DB), these drive the
//! fire-and-forget async methods — `connect`, `load_schemas`, `load_rows`,
//! `load_structure`, `run_query`, `commit_changes` — against a real PostgreSQL
//! container and pump the event loop until each `AppEvent` lands. This is the
//! only way to exercise the spawn closures and `apply_event` success paths.
//!
//! Requires Docker. Gated behind the `e2e` feature so default `cargo test`
//! stays infra-free:  `cargo test --features e2e`.

use std::time::{Duration, Instant};

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use yssv::app::YssvApp;
use yssv::pages::explorer::state::TabView;

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Pump the event channel until `pred` holds or the timeout elapses.
fn wait_until(
    app: &mut YssvApp,
    ctx: &egui::Context,
    label: &str,
    pred: impl Fn(&YssvApp) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        app.drain_events(ctx);
        if pred(app) {
            return;
        }
        if Instant::now() > deadline {
            panic!("timed out waiting for: {label}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn active_query_id(app: &YssvApp) -> String {
    app.explorer
        .as_ref()
        .unwrap()
        .tabs
        .active_query_tab()
        .unwrap()
        .id
        .clone()
}

#[test]
fn full_app_flow_against_postgres() {
    let runtime = rt();
    // Hold a runtime context for the whole test so the container's async Drop
    // (which stops/removes it) has a reactor to run on at teardown.
    let _guard = runtime.enter();
    let container = runtime.block_on(async { Postgres::default().start().await.unwrap() });
    let port = runtime.block_on(async { container.get_host_port_ipv4(5432).await.unwrap() });

    let ctx = egui::Context::default();
    let mut app = YssvApp::new_for_test();
    app.conn_page.form.host = "127.0.0.1".into();
    app.conn_page.form.port = port.to_string();
    app.conn_page.form.database = "postgres".into();
    app.conn_page.form.username = "postgres".into();
    app.conn_page.form.password = "postgres".into();

    // --- test_connection: TestOk path ---
    app.test_connection(ctx.clone());
    wait_until(&mut app, &ctx, "TestOk", |a| {
        matches!(
            a.conn_page.test_status,
            yssv::pages::connections::state::TestStatus::Ok(_)
        )
    });

    // --- connect: Connected path + auto load_schemas ---
    app.connect(ctx.clone());
    wait_until(&mut app, &ctx, "Connected", |a| a.explorer.is_some());
    wait_until(&mut app, &ctx, "schemas loaded", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.databases.iter().find(|d| d.name == "postgres"))
            .map(|d| !d.schemas.is_empty())
            .unwrap_or(false)
    });

    // --- run_query: CREATE / INSERT / SELECT (QueryExecuted) ---
    let queries = [
        "CREATE TABLE e2e_items (id int PRIMARY KEY, name text)",
        "INSERT INTO e2e_items (id, name) VALUES (1, 'alpha'), (2, 'beta')",
        "SELECT * FROM e2e_items ORDER BY id",
    ];
    for sql in queries {
        let tab_id = {
            let e = app.explorer.as_mut().unwrap();
            let q = e.tabs.open_query("postgres");
            q.sql = sql.to_string();
            q.id.clone()
        };
        app.run_query(
            ctx.clone(),
            tab_id.clone(),
            sql.to_string(),
            "postgres".into(),
        );
        wait_until(&mut app, &ctx, sql, |a| {
            a.explorer
                .as_ref()
                .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == tab_id))
                .and_then(|t| t.as_query())
                .map(|q| !q.loading && q.result.is_some())
                .unwrap_or(false)
        });
    }

    // --- run_query error path (QueryError) ---
    let bad_id = {
        let e = app.explorer.as_mut().unwrap();
        let q = e.tabs.open_query("postgres");
        q.sql = "SELECT * FROM table_that_does_not_exist".into();
        q.id.clone()
    };
    app.run_query(
        ctx.clone(),
        bad_id.clone(),
        "SELECT * FROM table_that_does_not_exist".into(),
        "postgres".into(),
    );
    wait_until(&mut app, &ctx, "QueryError", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == bad_id))
            .and_then(|t| t.as_query())
            .map(|q| q.error.is_some())
            .unwrap_or(false)
    });
    let _ = active_query_id(&app); // touch the accessor

    // --- load_rows on a real table (RowsLoaded) ---
    let table_tab_id = {
        let e = app.explorer.as_mut().unwrap();
        let tab = e.tabs.open_table("e2e_items", "public", "postgres");
        tab.id.clone()
    };
    app.load_rows(
        ctx.clone(),
        table_tab_id.clone(),
        "postgres".into(),
        "public".into(),
        "e2e_items".into(),
        100,
        0,
    );
    wait_until(&mut app, &ctx, "RowsLoaded", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == table_tab_id))
            .and_then(|t| t.as_table())
            .map(|t| t.result.is_some())
            .unwrap_or(false)
    });

    // --- load_structure (StructureLoaded) ---
    app.load_structure(
        ctx.clone(),
        table_tab_id.clone(),
        "postgres".into(),
        "public".into(),
        "e2e_items".into(),
    );
    wait_until(&mut app, &ctx, "StructureLoaded", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == table_tab_id))
            .and_then(|t| t.as_table())
            .map(|t| t.structure.is_some())
            .unwrap_or(false)
    });

    // --- commit_changes: queue an update + insert, commit, reload (CommitDone) ---
    {
        let e = app.explorer.as_mut().unwrap();
        // Make the table tab active so commit_changes targets it.
        let pos = e
            .tabs
            .tabs
            .iter()
            .position(|t| t.id() == table_tab_id)
            .unwrap();
        e.tabs.active = pos;
        let tab = e.tabs.active_table_tab_mut().unwrap();
        tab.view = TabView::Data;
        // Update row 0's name column (col index 1) and add a fresh insert row.
        tab.edits
            .updates
            .insert((0, 1), Some("alpha-edited".into()));
        tab.edits
            .inserts
            .push(vec![Some("3".into()), Some("gamma".into())]);
    }
    app.commit_changes(ctx.clone(), table_tab_id.clone());
    wait_until(&mut app, &ctx, "CommitDone reload", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == table_tab_id))
            .and_then(|t| t.as_table())
            .map(|t| t.result.is_some() && t.edits.is_empty() && !t.committing)
            .unwrap_or(false)
    });

    // The committed insert means the table now has three rows.
    let row_count = app
        .explorer
        .as_ref()
        .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == table_tab_id))
        .and_then(|t| t.as_table())
        .and_then(|t| t.result.as_ref())
        .map(|r| r.rows.len())
        .unwrap_or(0);
    assert_eq!(row_count, 3, "commit should have inserted a third row");

    // --- commit_failed path: an edit that violates the PK constraint ---
    {
        let e = app.explorer.as_mut().unwrap();
        let tab = e.tabs.active_table_tab_mut().unwrap();
        // Duplicate an existing primary key to force a unique violation.
        tab.edits
            .inserts
            .push(vec![Some("1".into()), Some("dup".into())]);
    }
    app.commit_changes(ctx.clone(), table_tab_id.clone());
    wait_until(&mut app, &ctx, "CommitFailed", |a| a.error_modal.is_some());

    // --- load_schemas explicitly (SchemasLoaded) ---
    app.load_schemas(ctx.clone(), "postgres".into());
    wait_until(&mut app, &ctx, "SchemasLoaded again", |a| {
        a.explorer
            .as_ref()
            .and_then(|e| e.databases.iter().find(|d| d.name == "postgres"))
            .map(|d| d.schemas.iter().any(|s| s.name == "public"))
            .unwrap_or(false)
    });

    // Sanity: a Tab::Table label and id accessor on the live tab.
    let label = app
        .explorer
        .as_ref()
        .and_then(|e| e.tabs.tabs.iter().find(|t| t.id() == table_tab_id))
        .map(|t| t.label())
        .unwrap_or_default();
    assert!(label.contains("e2e_items"));
}

#[test]
fn connect_to_unreachable_host_sets_error_modal() {
    let ctx = egui::Context::default();
    let mut app = YssvApp::new_for_test();
    app.conn_page.form.host = "127.0.0.1".into();
    // Port 1 is privileged/closed — connection is refused fast.
    app.conn_page.form.port = "1".into();
    app.conn_page.form.database = "postgres".into();
    app.conn_page.form.username = "postgres".into();
    app.conn_page.form.password = "postgres".into();

    app.connect(ctx.clone());
    wait_until(&mut app, &ctx, "ConnectError", |a| a.error_modal.is_some());
    assert!(app.explorer.is_none());
}
