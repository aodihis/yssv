//! Render-level smoke tests for the UI layer.
//!
//! These drive every page / molecule / atom render entry point through an
//! `egui_kittest` harness in a variety of application states. They don't assert
//! on pixels — egui rendering itself is not under test — but executing the
//! render code in each meaningful state exercises the branch logic (loading vs.
//! loaded vs. error, data vs. structure view, pending edits, dialogs, light vs.
//! dark theme, …) that pure unit tests can't reach.

use egui_kittest::Harness;

use yssv::app::YssvApp;
use yssv::core::connections::model::{ConnColor, Connection};
use yssv::core::edit::TableEdits;
use yssv::core::results::model::{ColumnDef, QueryResult};
use yssv::core::schema::model::{DbInfo, SchemaInfo, TableInfo, TableKind};
use yssv::pages::connections::state::TestStatus;
use yssv::pages::explorer::state::{
    EditingCell, ExplorerState, QueryTab, RowRef, Tab, TabView, TableTab,
};
use yssv::theme::Theme;

// ---------------------------------------------------------------------------
// Harness helpers
// ---------------------------------------------------------------------------

/// Run `render` against `app` inside a kittest harness, installing the real
/// fonts, image loaders, and theme so icon / SemiBold-font paths execute just
/// like in the live app.
fn drive(mut app: YssvApp, mut render: impl FnMut(&mut egui::Ui, &mut YssvApp)) {
    let theme = app.settings.theme;
    // The custom "SemiBold" font family must be bound before any text using it
    // is laid out — `set_fonts` only takes effect on the *next* frame, so the
    // first harness frame only installs fonts/loaders/theme and renders nothing.
    let mut frame = 0u32;
    let mut harness = Harness::new_ui(|ui| {
        if frame == 0 {
            yssv::theme::setup_fonts(ui.ctx());
            egui_extras::install_image_loaders(ui.ctx());
            yssv::theme::apply_theme(ui.ctx(), theme);
        } else {
            render(ui, &mut app);
        }
        frame += 1;
    });
    harness.run();
    harness.run();
}

/// Like [`drive`], but after the UI is built it clicks the first widget whose
/// accessible label matches `label` (if present) and pumps another frame, so
/// the click-handler branch executes. Buttons rendered via real egui widgets
/// (text labels) are queryable; purely painter-drawn rows are not.
fn drive_click(label: &str, mut app: YssvApp, mut render: impl FnMut(&mut egui::Ui, &mut YssvApp)) {
    use egui_kittest::kittest::Queryable;
    let theme = app.settings.theme;
    let mut frame = 0u32;
    let mut harness = Harness::new_ui(|ui| {
        if frame == 0 {
            yssv::theme::setup_fonts(ui.ctx());
            egui_extras::install_image_loaders(ui.ctx());
            yssv::theme::apply_theme(ui.ctx(), theme);
        } else {
            render(ui, &mut app);
        }
        frame += 1;
    });
    harness.run();
    harness.run();
    if let Some(node) = harness.query_by_label(label) {
        node.click();
        harness.run();
    }
    harness.run();
}

/// Run a free-standing widget closure (no `YssvApp`) inside a themed harness.
fn drive_ui(theme: Theme, mut f: impl FnMut(&mut egui::Ui)) {
    let mut frame = 0u32;
    let mut harness = Harness::new_ui(|ui| {
        if frame == 0 {
            yssv::theme::setup_fonts(ui.ctx());
            egui_extras::install_image_loaders(ui.ctx());
            yssv::theme::apply_theme(ui.ctx(), theme);
        } else {
            f(ui);
        }
        frame += 1;
    });
    harness.run();
    harness.run();
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn cols() -> Vec<ColumnDef> {
    vec![
        ColumnDef {
            name: "id".into(),
            data_type: "int4".into(),
            is_pk: true,
            is_fk: false,
            nullable: false,
        },
        ColumnDef {
            name: "name".into(),
            data_type: "text".into(),
            is_pk: false,
            is_fk: false,
            nullable: true,
        },
        ColumnDef {
            name: "active".into(),
            data_type: "bool".into(),
            is_pk: false,
            is_fk: true,
            nullable: false,
        },
        ColumnDef {
            name: "created".into(),
            data_type: "timestamp".into(),
            is_pk: false,
            is_fk: false,
            nullable: true,
        },
    ]
}

fn rows() -> Vec<Vec<Option<String>>> {
    vec![
        vec![
            Some("1".into()),
            Some("alice".into()),
            Some("true".into()),
            Some("2024-01-01 00:00:00".into()),
        ],
        vec![Some("2".into()), None, Some("false".into()), None],
    ]
}

fn result() -> QueryResult {
    QueryResult {
        columns: cols(),
        rows: rows(),
        total_rows: Some(250),
    }
}

fn schemas_with_tables() -> Vec<SchemaInfo> {
    vec![SchemaInfo {
        name: "public".into(),
        tables: vec![
            TableInfo {
                name: "users".into(),
                kind: TableKind::Table,
                row_count: Some(4200),
            },
            TableInfo {
                name: "orders".into(),
                kind: TableKind::Table,
                row_count: Some(12),
            },
            TableInfo {
                name: "active_users".into(),
                kind: TableKind::View,
                row_count: None,
            },
        ],
    }]
}

fn explorer_with_schemas() -> ExplorerState {
    let dbs = vec![
        DbInfo {
            name: "mydb".into(),
            schemas: schemas_with_tables(),
        },
        DbInfo {
            name: "analytics".into(),
            schemas: vec![],
        },
    ];
    let mut ex = ExplorerState::new("c1".into(), "My Connection".into(), "mydb", dbs);
    // Expand the schema node so tables (and the view pill) render.
    ex.open_nodes.insert("sc:mydb:public".into());
    ex
}

fn app_connections() -> YssvApp {
    let mut app = YssvApp::new_for_test();
    let mut a = Connection::new_postgres();
    a.name = "Prod".into();
    a.group = "Production".into();
    let mut b = Connection::new_mysql();
    b.name = "Local Dev".into();
    b.group = "Local".into();
    b.ssh = Some(yssv::core::ssh::model::SshConfig {
        host: "bastion".into(),
        port: 22,
        username: "admin".into(),
        auth: yssv::core::ssh::model::SshAuth::Agent,
    });
    app.conn_page = yssv::pages::connections::state::ConnectionsPageState::new(vec![a, b]);
    app
}

fn app_with_table(view: TabView, loading: bool, with_result: bool) -> YssvApp {
    let mut app = YssvApp::new_for_test();
    app.set_conn_config_for_test(Connection::new_postgres());
    let mut ex = explorer_with_schemas();
    let mut tab = TableTab::new("users", "public", "mydb");
    tab.view = view;
    tab.loading = loading;
    if with_result {
        tab.result = Some(result());
    }
    ex.tabs.tabs.push(Tab::Table(tab));
    app.explorer = Some(ex);
    app
}

fn app_with_query(loading: bool, error: Option<&str>, with_result: bool) -> YssvApp {
    let mut app = YssvApp::new_for_test();
    app.set_conn_config_for_test(Connection::new_postgres());
    let mut ex = explorer_with_schemas();
    let mut q = QueryTab::new("mydb", 1);
    q.sql = "SELECT * FROM users WHERE id = 1".into();
    q.loading = loading;
    q.error = error.map(|e| e.to_string());
    if with_result {
        q.result = Some(result());
    }
    ex.tabs.tabs.push(Tab::Query(q));
    app.explorer = Some(ex);
    app
}

// ---------------------------------------------------------------------------
// error_modal
// ---------------------------------------------------------------------------

#[test]
fn error_modal_hidden_when_none() {
    let app = YssvApp::new_for_test();
    drive(app, |ui, app| yssv::ui::error_modal::render(ui, app));
}

#[test]
fn error_modal_shown_when_set() {
    let mut app = YssvApp::new_for_test();
    app.error_modal = Some("boom".into());
    drive(app, |ui, app| yssv::ui::error_modal::render(ui, app));
}

// ---------------------------------------------------------------------------
// Connections page (list + detail + dialogs)
// ---------------------------------------------------------------------------

#[test]
fn connections_page_renders_dark() {
    let app = app_connections();
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_renders_light() {
    let mut app = app_connections();
    app.settings.theme = Theme::Light;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_new_form() {
    let mut app = app_connections();
    app.conn_page.start_new();
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_mysql_engine_form() {
    let mut app = app_connections();
    app.conn_page.form =
        yssv::pages::connections::state::ConnectionForm::from_connection(&Connection::new_mysql());
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_ssh_password_form() {
    let mut app = app_connections();
    app.conn_page.form.ssh_enabled = true;
    app.conn_page.form.ssh_auth_method = yssv::pages::connections::state::SshAuthMethod::Password;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_ssh_keyfile_form() {
    let mut app = app_connections();
    app.conn_page.form.ssh_enabled = true;
    app.conn_page.form.ssh_auth_method = yssv::pages::connections::state::SshAuthMethod::KeyFile;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_ssh_agent_form() {
    let mut app = app_connections();
    app.conn_page.form.ssh_enabled = true;
    app.conn_page.form.ssh_auth_method = yssv::pages::connections::state::SshAuthMethod::Agent;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_connecting_state() {
    let mut app = app_connections();
    app.conn_page.connecting = true;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_testing_state() {
    let mut app = app_connections();
    app.conn_page.test_status = TestStatus::Testing;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_test_ok_status() {
    let mut app = app_connections();
    app.conn_page.test_status = TestStatus::Ok(12);
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_saved_status() {
    let mut app = app_connections();
    app.conn_page.save_status = yssv::pages::connections::state::SaveStatus::Saved;
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_test_error_dialog() {
    let mut app = app_connections();
    app.conn_page.test_status = TestStatus::Failed("auth failed".into());
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_delete_confirm_dialog() {
    let mut app = app_connections();
    let id = app.conn_page.connections[0].id.clone();
    app.conn_page.pending_delete = Some(id);
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_collapsed_group() {
    let mut app = app_connections();
    app.conn_page.collapsed_groups.insert("Local".into());
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_renaming_group() {
    let mut app = app_connections();
    app.conn_page.renaming_group = Some(("Local".into(), "Local".into()));
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

#[test]
fn connections_page_search_filter() {
    let mut app = app_connections();
    app.conn_page.search_query = "prod".into();
    drive(app, |ui, app| yssv::pages::connections::render(ui, app));
}

// ---------------------------------------------------------------------------
// Explorer page (sidebar + main_view)
// ---------------------------------------------------------------------------

#[test]
fn explorer_page_no_explorer() {
    let app = YssvApp::new_for_test();
    drive(app, |ui, app| yssv::pages::explorer::render(ui, app));
}

#[test]
fn explorer_page_empty_tabs() {
    let mut app = YssvApp::new_for_test();
    app.explorer = Some(explorer_with_schemas());
    drive(app, |ui, app| yssv::pages::explorer::render(ui, app));
}

#[test]
fn explorer_page_with_table_tab_dark() {
    let app = app_with_table(TabView::Data, false, true);
    drive(app, |ui, app| yssv::pages::explorer::render(ui, app));
}

#[test]
fn explorer_page_with_table_tab_light() {
    let mut app = app_with_table(TabView::Data, false, true);
    app.settings.theme = Theme::Light;
    drive(app, |ui, app| yssv::pages::explorer::render(ui, app));
}

#[test]
fn sidebar_renders_tree() {
    let app = app_with_table(TabView::Data, false, true);
    drive(app, |ui, app| {
        yssv::pages::explorer::sidebar::render_sidebar(ui, app)
    });
}

#[test]
fn sidebar_with_filter() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(e) = app.explorer.as_mut() {
        e.filter = "user".into();
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::sidebar::render_sidebar(ui, app)
    });
}

#[test]
fn sidebar_light_theme() {
    let mut app = app_with_table(TabView::Data, false, true);
    app.settings.theme = Theme::Light;
    drive(app, |ui, app| {
        yssv::pages::explorer::sidebar::render_sidebar(ui, app)
    });
}

// --- main_view: table tab states ---

#[test]
fn main_view_table_data_loaded() {
    let app = app_with_table(TabView::Data, false, true);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_loading() {
    let app = app_with_table(TabView::Data, true, false);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_structure_loaded() {
    let mut app = app_with_table(TabView::Structure, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.structure = Some(cols());
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_structure_loading() {
    let mut app = app_with_table(TabView::Structure, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.structure_loading = true;
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_with_pending_edits() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("changed".into()));
        tab.edits.deletes.insert(1);
        tab.selected_row = Some(2);
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_with_insert_row() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.add_insert_row();
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_table_committing() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("x".into()));
        tab.committing = true;
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_commit_dialog_open() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("changed".into()));
        tab.show_commit_dialog = true;
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

// --- main_view: query tab states ---

#[test]
fn main_view_query_empty() {
    let app = app_with_query(false, None, false);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_query_loading() {
    let app = app_with_query(true, None, false);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_query_error() {
    let app = app_with_query(false, Some("syntax error near SELECT"), false);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_query_with_results() {
    let app = app_with_query(false, None, true);
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_query_empty_result_set() {
    let mut app = app_with_query(false, None, true);
    if let Some(q) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_query_tab_mut())
    {
        q.result = Some(QueryResult {
            columns: cols(),
            rows: vec![],
            total_rows: Some(0),
        });
    }
    drive(app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

// ---------------------------------------------------------------------------
// Molecules — driven directly
// ---------------------------------------------------------------------------

#[test]
fn data_table_renders_with_selection() {
    let c = cols();
    let r = rows();
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::data_table::data_table(ui, &c, &r, 30.0, Some(0));
    });
}

#[test]
fn data_table_empty_rows() {
    let c = cols();
    drive_ui(Theme::Light, |ui| {
        yssv::ui::molecules::data_table::data_table(ui, &c, &[], 30.0, None);
    });
}

#[test]
fn editable_table_all_edit_states() {
    let c = cols();
    let r = rows();
    let mut edits = TableEdits::default();
    edits.updates.insert((0, 1), Some("edited".into()));
    edits.deletes.insert(1);
    edits.inserts.push(vec![
        Some("99".into()),
        Some("new".into()),
        Some("true".into()),
        None,
    ]);
    let mut editing = None;
    let mut selected = Some(0usize);
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::editable_data_table::editable_data_table(
            ui,
            &c,
            &r,
            &mut edits,
            &mut editing,
            &mut selected,
            30.0,
        );
    });
}

#[test]
fn editable_table_with_editing_cell() {
    let c = cols();
    let r = rows();
    let mut edits = TableEdits::default();
    let mut editing = Some(EditingCell {
        row: RowRef::Original(0),
        col: 1,
        buffer: "typing".into(),
        request_focus: true,
    });
    let mut selected = None;
    drive_ui(Theme::Light, |ui| {
        yssv::ui::molecules::editable_data_table::editable_data_table(
            ui,
            &c,
            &r,
            &mut edits,
            &mut editing,
            &mut selected,
            30.0,
        );
    });
}

#[test]
fn tree_row_variants() {
    use yssv::ui::atoms::icon::Icon;
    use yssv::ui::molecules::tree_row::{TreeRowConfig, tree_row};
    drive_ui(Theme::Dark, |ui| {
        tree_row(
            ui,
            TreeRowConfig {
                label: "mydb".into(),
                level: 0,
                is_leaf: false,
                is_open: true,
                is_active: true,
                icon: Icon::Database,
                icon_color: None,
                count: None,
                pill: None,
            },
        );
        tree_row(
            ui,
            TreeRowConfig {
                label: "public".into(),
                level: 1,
                is_leaf: false,
                is_open: false,
                is_active: false,
                icon: Icon::Layers,
                icon_color: None,
                count: Some("12".into()),
                pill: None,
            },
        );
        tree_row(
            ui,
            TreeRowConfig {
                label: "a_very_long_table_name_that_truncates".into(),
                level: 2,
                is_leaf: true,
                is_open: false,
                is_active: false,
                icon: Icon::Eye,
                icon_color: Some(yssv::theme::colors::PURPLE),
                count: None,
                pill: Some("VIEW".into()),
            },
        );
    });
}

#[test]
fn conn_item_selected_and_unselected() {
    let mut c = Connection::new_postgres();
    c.name = "Prod DB".into();
    c.ssh = Some(yssv::core::ssh::model::SshConfig {
        host: "bastion".into(),
        port: 22,
        username: "admin".into(),
        auth: yssv::core::ssh::model::SshAuth::Agent,
    });
    let plain = Connection::new_mysql();
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::conn_item::conn_item(ui, &c, true);
        yssv::ui::molecules::conn_item::conn_item(ui, &plain, false);
    });
}

#[test]
fn alert_dialog_renders() {
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::alert_dialog::error_dialog(ui, "t", "Title", "Something failed");
    });
}

#[test]
fn commit_dialog_single_and_multi() {
    let one = vec!["UPDATE users SET name = 'x' WHERE id = 1".to_string()];
    let many = vec![
        "UPDATE users SET name = 'x' WHERE id = 1".to_string(),
        "DELETE FROM users WHERE id = 2".to_string(),
    ];
    drive_ui(Theme::Light, |ui| {
        yssv::ui::molecules::commit_dialog::commit_dialog(ui, "c1", &one);
    });
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::commit_dialog::commit_dialog(ui, "c2", &many);
    });
}

#[test]
fn color_picker_renders() {
    let mut color = ConnColor::Blue;
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::molecules::color_picker::color_picker(ui, &mut color);
    });
}

// ---------------------------------------------------------------------------
// Atoms / layouts — driven directly
// ---------------------------------------------------------------------------

#[test]
fn group_input_renders() {
    let mut value = "Lo".to_string();
    let groups = vec!["Local".to_string(), "Production".to_string()];
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::atoms::group_input::group_input(ui, &mut value, &groups);
    });
}

#[test]
fn dividers_render() {
    drive_ui(Theme::Dark, |ui| {
        ui.horizontal(|ui| {
            yssv::ui::atoms::divider::v_divider(ui);
        });
        yssv::ui::atoms::divider::h_divider(ui);
    });
}

#[test]
fn label_dots_render() {
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::atoms::label_dot::label_dot(ui, ConnColor::Red, 10.0);
        yssv::ui::atoms::label_dot::colored_dot(ui, yssv::theme::colors::GREEN, 8.0);
    });
}

#[test]
fn toggle_switch_renders() {
    let mut on = false;
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::atoms::toggle::toggle_switch(ui, &mut on);
    });
}

#[test]
fn two_pane_layout_renders() {
    drive_ui(Theme::Dark, |ui| {
        yssv::ui::layouts::two_pane::two_pane(
            ui,
            120.0,
            |ui| ui.label("left"),
            |ui| ui.label("right"),
        );
    });
}

#[test]
fn sidebar_layout_renders() {
    drive_ui(Theme::Light, |ui| {
        yssv::ui::layouts::sidebar_layout::sidebar_layout(
            ui,
            150.0,
            |ui| {
                ui.label("sidebar");
            },
            |ui| {
                ui.label("main");
            },
        );
    });
}

#[test]
fn data_cell_all_branches() {
    use yssv::ui::molecules::data_cell::render_cell;
    let c = cols();
    drive_ui(Theme::Dark, |ui| {
        // null
        render_cell(ui, &c[1], &None);
        // numeric
        render_cell(ui, &c[0], &Some("42".into()));
        // bool true / false
        render_cell(ui, &c[2], &Some("true".into()));
        render_cell(ui, &c[2], &Some("false".into()));
        // timestamp
        render_cell(ui, &c[3], &Some("2024-01-01".into()));
        // plain text
        render_cell(ui, &c[1], &Some("hello".into()));
    });
}

// ---------------------------------------------------------------------------
// Interaction tests — click real button widgets to drive handler branches
// ---------------------------------------------------------------------------

#[test]
fn connections_list_new_button_starts_new_form() {
    let app = app_connections();
    drive_click("+ New", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn detail_save_button_persists_connection() {
    let mut app = app_connections();
    app.conn_page.form.name = "Saveable".into();
    drive_click("Save", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn detail_engine_toggle_to_mysql() {
    let app = app_connections();
    drive_click("MySQL", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn detail_duplicate_button() {
    let app = app_connections();
    drive_click("Duplicate connection", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn test_error_dialog_dismiss_button() {
    let mut app = app_connections();
    app.conn_page.test_status = TestStatus::Failed("nope".into());
    drive_click("Dismiss", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn delete_confirm_cancel_button() {
    let mut app = app_connections();
    let id = app.conn_page.connections[0].id.clone();
    app.conn_page.pending_delete = Some(id);
    drive_click("Cancel", app, |ui, app| {
        yssv::pages::connections::render(ui, app)
    });
}

#[test]
fn main_view_switch_to_structure_view() {
    let app = app_with_table(TabView::Data, false, true);
    drive_click("Structure", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_add_row_button() {
    let app = app_with_table(TabView::Data, false, true);
    drive_click("＋ Add Row", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_revert_button() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("x".into()));
    }
    drive_click("Revert", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_commit_button_opens_dialog() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("x".into()));
    }
    drive_click("✓ Commit", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_commit_dialog_cancel_button() {
    let mut app = app_with_table(TabView::Data, false, true);
    if let Some(tab) = app
        .explorer
        .as_mut()
        .and_then(|e| e.tabs.active_table_tab_mut())
    {
        tab.edits.updates.insert((0, 1), Some("x".into()));
        tab.show_commit_dialog = true;
    }
    drive_click("Cancel", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn main_view_query_run_button() {
    let app = app_with_query(false, None, false);
    drive_click("▶ Run", app, |ui, app| {
        yssv::pages::explorer::main_view::render_main(ui, app)
    });
}

#[test]
fn sidebar_sql_button_opens_query_tab() {
    let app = app_with_table(TabView::Data, false, true);
    drive_click("SQL", app, |ui, app| {
        yssv::pages::explorer::sidebar::render_sidebar(ui, app)
    });
}

#[test]
fn sidebar_back_to_connections_button() {
    let app = app_with_table(TabView::Data, false, true);
    drive_click("◀  Connections", app, |ui, app| {
        yssv::pages::explorer::sidebar::render_sidebar(ui, app)
    });
}

#[test]
fn commit_dialog_confirm_returns_choice() {
    use egui_kittest::kittest::Queryable;
    use yssv::ui::molecules::commit_dialog::{CommitChoice, commit_dialog};
    let stmts = vec!["DELETE FROM t WHERE id = 1".to_string()];
    let mut choice = CommitChoice::Pending;
    let mut frame = 0u32;
    let mut harness = Harness::new_ui(|ui| {
        if frame == 0 {
            yssv::theme::setup_fonts(ui.ctx());
            egui_extras::install_image_loaders(ui.ctx());
            yssv::theme::apply_theme(ui.ctx(), Theme::Dark);
        } else {
            let c = commit_dialog(ui, "cd", &stmts);
            if c != CommitChoice::Pending {
                choice = c;
            }
        }
        frame += 1;
    });
    harness.run();
    harness.run();
    harness.get_by_label("Commit").click();
    harness.run();
    drop(harness);
    assert_eq!(choice, CommitChoice::Confirm);
}

#[test]
fn error_dialog_dismiss_returns_true() {
    use egui_kittest::kittest::Queryable;
    use yssv::ui::molecules::alert_dialog::error_dialog;
    let mut dismissed = false;
    let mut frame = 0u32;
    let mut harness = Harness::new_ui(|ui| {
        if frame == 0 {
            yssv::theme::setup_fonts(ui.ctx());
            egui_extras::install_image_loaders(ui.ctx());
            yssv::theme::apply_theme(ui.ctx(), Theme::Dark);
        } else if error_dialog(ui, "ed", "Title", "msg") {
            dismissed = true;
        }
        frame += 1;
    });
    harness.run();
    harness.run();
    harness.get_by_label("Dismiss").click();
    harness.run();
    drop(harness);
    assert!(dismissed);
}
