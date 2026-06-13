use egui_kittest::{Harness, kittest::Queryable};
use yssv::{
    core::connections::model::{Connection, DbEngine},
    pages::explorer::state::{Tab, TableTab},
    ui::{
        atoms::{
            badge::{count_pill, engine_badge, view_pill},
            button::{compact_button, primary_button},
            dropdown::dropdown,
            input::{mono_input, password_input, text_input},
            light_switch::light_switch,
        },
        molecules::{
            status_bar::status_bar,
            tab_bar::tab_bar,
        },
    },
};

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

#[test]
fn primary_button_has_correct_label() {
    let harness = Harness::new_ui(|ui| {
        primary_button(ui, "Connect");
    });
    harness.get_by_label("Connect");
}

#[test]
fn compact_button_has_correct_label() {
    let harness = Harness::new_ui(|ui| {
        compact_button(ui, "Save");
    });
    harness.get_by_label("Save");
}

#[test]
fn primary_button_click_fires_response() {
    let mut clicked = false;
    let mut harness = Harness::new_ui(|ui| {
        if primary_button(ui, "Go").clicked() {
            clicked = true;
        }
    });
    harness.get_by_label("Go").click();
    harness.run();
    drop(harness);
    assert!(clicked);
}

#[test]
fn compact_button_click_fires_response() {
    let mut clicked = false;
    let mut harness = Harness::new_ui(|ui| {
        if compact_button(ui, "Apply").clicked() {
            clicked = true;
        }
    });
    harness.get_by_label("Apply").click();
    harness.run();
    drop(harness);
    assert!(clicked);
}

// ---------------------------------------------------------------------------
// Text inputs
// ---------------------------------------------------------------------------

#[test]
fn text_input_renders_without_panic() {
    let mut value = String::new();
    let harness = Harness::new_ui(|ui| {
        text_input(ui, &mut value, "Enter host", None);
    });
    drop(harness);
}

#[test]
fn text_input_with_icon_renders_without_panic() {
    let mut value = String::new();
    let harness = Harness::new_ui(|ui| {
        text_input(ui, &mut value, "Username", Some("@"));
    });
    drop(harness);
}

#[test]
fn text_input_with_prefilled_value_renders_without_panic() {
    let mut value = "localhost".to_string();
    let harness = Harness::new_ui(|ui| {
        text_input(ui, &mut value, "Host", None);
    });
    drop(harness);
}

#[test]
fn password_input_renders_without_panic() {
    let mut value = String::new();
    let harness = Harness::new_ui(|ui| {
        password_input(ui, &mut value, "Password", None);
    });
    drop(harness);
}

#[test]
fn mono_input_renders_without_panic() {
    let mut value = "SELECT * FROM users".to_string();
    let harness = Harness::new_ui(|ui| {
        mono_input(ui, &mut value, "SQL", None);
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Light switch (custom painted — smoke tests)
// ---------------------------------------------------------------------------

#[test]
fn light_switch_off_renders_without_panic() {
    let mut on = false;
    let harness = Harness::new_ui(|ui| {
        light_switch(ui, &mut on);
    });
    drop(harness);
}

#[test]
fn light_switch_on_renders_without_panic() {
    let mut on = true;
    let harness = Harness::new_ui(|ui| {
        light_switch(ui, &mut on);
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Badges (custom painted — smoke tests only)
// ---------------------------------------------------------------------------

#[test]
fn engine_badge_postgres_renders_without_panic() {
    let harness = Harness::new_ui(|ui| {
        engine_badge(ui, DbEngine::Postgres);
    });
    drop(harness);
}

#[test]
fn engine_badge_mysql_renders_without_panic() {
    let harness = Harness::new_ui(|ui| {
        engine_badge(ui, DbEngine::MySQL);
    });
    drop(harness);
}

#[test]
fn view_pill_renders_without_panic() {
    let harness = Harness::new_ui(|ui| {
        view_pill(ui);
    });
    drop(harness);
}

#[test]
fn count_pill_renders_without_panic() {
    let harness = Harness::new_ui(|ui| {
        count_pill(ui, "1.2k");
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Dropdown
// ---------------------------------------------------------------------------

#[test]
fn dropdown_with_selection_renders_without_panic() {
    let mut selected = "pg".to_string();
    let options = vec![("pg".to_string(), "PostgreSQL"), ("my".to_string(), "MySQL")];
    let harness = Harness::new_ui(|ui| {
        dropdown(ui, "test_db_type", &mut selected, &options);
    });
    drop(harness);
}

#[test]
fn dropdown_with_no_match_renders_without_panic() {
    let mut selected = "unknown".to_string();
    let options = vec![("pg".to_string(), "PostgreSQL"), ("my".to_string(), "MySQL")];
    let harness = Harness::new_ui(|ui| {
        dropdown(ui, "test_unmatched", &mut selected, &options);
    });
    drop(harness);
}

#[test]
fn dropdown_empty_options_renders_without_panic() {
    let mut selected = String::new();
    let options: Vec<(String, &str)> = vec![];
    let harness = Harness::new_ui(|ui| {
        dropdown(ui, "test_empty", &mut selected, &options);
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Molecules — tab_bar (custom painted — smoke tests)
// ---------------------------------------------------------------------------

#[test]
fn tab_bar_empty_renders_without_panic() {
    let harness = Harness::new_ui(|ui| {
        tab_bar(ui, &[], 0);
    });
    drop(harness);
}

#[test]
fn tab_bar_single_tab_renders_without_panic() {
    let tabs = vec![Tab::Table(TableTab::new("users", "public", "mydb"))];
    let harness = Harness::new_ui(|ui| {
        tab_bar(ui, &tabs, 0);
    });
    drop(harness);
}

#[test]
fn tab_bar_multiple_tabs_renders_without_panic() {
    let tabs = vec![
        Tab::Table(TableTab::new("users", "public", "mydb")),
        Tab::Table(TableTab::new("orders", "public", "mydb")),
        Tab::Table(TableTab::new("products", "public", "mydb")),
    ];
    let harness = Harness::new_ui(|ui| {
        tab_bar(ui, &tabs, 1);
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Molecules — status_bar
// ---------------------------------------------------------------------------

#[test]
fn status_bar_renders_without_panic() {
    let tab = TableTab::new("users", "public", "mydb");
    let harness = Harness::new_ui(|ui| {
        status_bar(ui, &tab);
    });
    drop(harness);
}

#[test]
fn status_bar_with_loaded_rows_renders_without_panic() {
    use yssv::core::results::model::{ColumnDef, QueryResult};
    let mut tab = TableTab::new("orders", "public", "mydb");
    tab.result = Some(QueryResult {
        columns: vec![ColumnDef {
            name: "id".into(),
            data_type: "int4".into(),
            is_pk: true,
            is_fk: false,
            nullable: false,
        }],
        rows: vec![vec![Some("1".into())]],
        total_rows: Some(42),
    });
    let harness = Harness::new_ui(|ui| {
        status_bar(ui, &tab);
    });
    drop(harness);
}

// ---------------------------------------------------------------------------
// Connection form (uses atoms together)
// ---------------------------------------------------------------------------

#[test]
fn connection_form_atoms_render_together() {
    let mut conn = Connection::new_postgres();
    let harness = Harness::new_ui(|ui| {
        text_input(ui, &mut conn.name, "Connection name", None);
        text_input(ui, &mut conn.host, "Host", None);
        password_input(ui, &mut conn.password, "Password", None);
        primary_button(ui, "Connect");
    });
    harness.get_by_label("Connect");
    drop(harness);
}
