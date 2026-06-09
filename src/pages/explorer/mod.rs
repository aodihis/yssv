pub mod state;
pub use state::{ExplorerState, TabState, TabView, TableTab};

use crate::core::schema::model::TableKind;

type TableRow = (String, TableKind, Option<u64>, bool);
type SchemaRow = (String, bool, Vec<TableRow>);
type DbRow = (String, bool, bool, Vec<SchemaRow>);

#[derive(Debug)]
struct LoadRequest {
    tab_id: String,
    db: String,
    schema: String,
    table: String,
    limit: u32,
    offset: u32,
}

pub fn render_sidebar(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    use crate::theme::colors;
    use crate::ui::molecules::tree_row::{tree_row, TreeRowConfig};
    use egui::RichText;

    use crate::theme::ThemeColors;
    if app.explorer.is_none() {
        return;
    }
    let accent_color = ui.visuals().selection.stroke.color;
    let tc = ThemeColors::from_ui(ui);

    // Header — 10px 10px 8px padding, border-bottom-faint (design spec)
    let conn_name = app.explorer.as_ref().unwrap().conn_name.clone();

    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 10,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.label(RichText::new(&conn_name).size(13.0).strong().color(tc.foreground));
            ui.add_space(7.0);
            let e = app.explorer.as_mut().unwrap();
            egui::TextEdit::singleline(&mut e.filter)
                .hint_text("Filter tables…")
                .desired_width(ui.available_width())
                .show(ui);
        });

    // Border below sidebar head
    let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
    ui.painter()
        .rect_filled(sep, egui::CornerRadius::ZERO, tc.border_muted);
    ui.add_space(1.0);

    // Collect what to render from explorer (immutable snapshot)
    let tree_data: Vec<DbRow> = {
        let e = app.explorer.as_ref().unwrap();
        let q = e.filter.to_lowercase();
        e.databases
            .iter()
            .map(|db| {
                let db_key = format!("db:{}", db.name);
                let db_open = e.is_open(&db_key);
                let is_active = e.active_db == db.name;
                let schemas = if db_open {
                    db.schemas
                        .iter()
                        .map(|sc| {
                            let sc_key = format!("sc:{}:{}", db.name, sc.name);
                            let sc_open = e.is_open(&sc_key);
                            let tables = if sc_open {
                                sc.tables
                                    .iter()
                                    .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q))
                                    .map(|t| {
                                        let active = e
                                            .tabs
                                            .active_tab()
                                            .map(|tab| {
                                                tab.table == t.name
                                                    && tab.schema == sc.name
                                                    && tab.database == db.name
                                            })
                                            .unwrap_or(false);
                                        (t.name.clone(), t.kind, t.row_count, active)
                                    })
                                    .collect()
                            } else {
                                vec![]
                            };
                            (sc.name.clone(), sc_open, tables)
                        })
                        .collect()
                } else {
                    vec![]
                };
                (db.name.clone(), db_open, is_active, schemas)
            })
            .collect()
    };

    // Now render the tree and collect mutations
    let mut toggle_node: Option<String> = None;
    let mut open_table: Option<(String, String, String)> = None; // (table, schema, db)

    let footer_h = 28.0;
    let available = ui.available_rect_before_wrap();
    let tree_rect = egui::Rect::from_min_size(
        available.min,
        egui::vec2(available.width(), (available.height() - footer_h).max(0.0)),
    );
    let footer_rect = egui::Rect::from_min_size(
        egui::pos2(available.min.x, available.max.y - footer_h),
        egui::vec2(available.width(), footer_h),
    );

    let mut tree_ui = ui.new_child(egui::UiBuilder::new().max_rect(tree_rect));
    egui::ScrollArea::vertical().show(&mut tree_ui, |ui| {
        ui.add_space(6.0);
        for (db_name, db_open, is_active, schemas) in &tree_data {
            let db_key = format!("db:{}", db_name);
            let resp = tree_row(
                ui,
                TreeRowConfig {
                    label: db_name,
                    level: 0,
                    is_leaf: false,
                    is_open: *db_open,
                    is_active: *is_active,
                    icon: "🗄",
                    icon_color: if *is_active { Some(accent_color) } else { None },
                    count: None,
                    pill: None,
                },
            );
            if resp.clicked() {
                toggle_node = Some(db_key);
            }

            for (schema_name, sc_open, tables) in schemas {
                let sc_key = format!("sc:{}:{}", db_name, schema_name);
                let resp = tree_row(
                    ui,
                    TreeRowConfig {
                        label: schema_name,
                        level: 1,
                        is_leaf: false,
                        is_open: *sc_open,
                        is_active: false,
                        icon: "◫",
                        icon_color: None,
                        count: Some(tables.len().to_string()),
                        pill: None,
                    },
                );
                if resp.clicked() {
                    toggle_node = Some(sc_key);
                }

                for (table_name, kind, row_count, active_table) in tables {
                    let is_view = *kind == TableKind::View;
                    let count_str = row_count.map(|n| {
                        if n >= 1000 {
                            format!("{:.0}k", n as f64 / 1000.0)
                        } else {
                            n.to_string()
                        }
                    });
                    let resp = tree_row(
                        ui,
                        TreeRowConfig {
                            label: table_name,
                            level: 2,
                            is_leaf: true,
                            is_open: false,
                            is_active: *active_table,
                            icon: if is_view { "⊡" } else { "▦" },
                            icon_color: if is_view { Some(colors::PURPLE) } else { None },
                            count: if is_view { None } else { count_str },
                            pill: if is_view { Some("VIEW") } else { None },
                        },
                    );
                    if resp.clicked() {
                        open_table =
                            Some((table_name.clone(), schema_name.clone(), db_name.clone()));
                    }
                }
            }
        }
    });

    // Apply mutations after rendering (borrow is released)
    if let Some(key) = toggle_node {
        if let Some(e) = &mut app.explorer {
            e.toggle_node(&key);
        }
    }
    if let Some((table, schema, db)) = open_table {
        let load_req = {
            let e = app.explorer.as_mut().unwrap();
            let tab = e.tabs.open(&table, &schema, &db);
            if tab.result.is_none() && !tab.loading {
                tab.loading = true;
                Some(LoadRequest {
                    tab_id: tab.id.clone(),
                    db: db.clone(),
                    schema: schema.clone(),
                    table: table.clone(),
                    limit: tab.page_size,
                    offset: tab.offset(),
                })
            } else {
                None
            }
        };
        if let Some(req) = load_req {
            app.load_rows(
                ctx.clone(),
                req.tab_id,
                req.db,
                req.schema,
                req.table,
                req.limit,
                req.offset,
            );
        }
    }

    // Footer
    let mut footer_ui = ui.new_child(egui::UiBuilder::new().max_rect(footer_rect));
    footer_ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        ui.colored_label(colors::SUCCESS, "●");
        ui.label(egui::RichText::new("Connected").size(11.0));
    });
}

pub fn render_main(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    use crate::pages::explorer::state::TabView;
    use crate::theme::ThemeColors;
    use crate::ui::molecules::{
        data_cell::render_cell,
        status_bar::{status_bar, STATUS_H},
        tab_bar::tab_bar,
    };
    use egui::RichText;

    if app.explorer.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a connection to explore").size(14.0));
        });
        return;
    }

    // Tab operations
    let (activate, close) = {
        let explorer = app.explorer.as_ref().unwrap();
        if explorer.tabs.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select a table from the sidebar")
                        .size(14.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });
            return;
        }
        tab_bar(ui, &explorer.tabs.tabs, explorer.tabs.active)
    };
    {
        let explorer = app.explorer.as_mut().unwrap();
        if let Some(i) = activate {
            explorer.tabs.active = i;
        }
        if let Some(i) = close {
            explorer.tabs.close(i);
        }
    }

    // Sub-view toggle toolbar — 32px height, border-bottom
    {
        let tc = ThemeColors::from_ui(ui);
        let toolbar_rect =
            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 32.0));
        ui.painter()
            .rect_filled(toolbar_rect, egui::CornerRadius::ZERO, tc.surface);

        let explorer = app.explorer.as_mut().unwrap();
        if let Some(tab) = explorer.tabs.active_tab() {
            let is_data = tab.view == TabView::Data;
            let mut new_view: Option<TabView> = None;
            ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar_rect), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    for (label, view, active) in [
                        ("Data", TabView::Data, is_data),
                        ("Structure", TabView::Structure, !is_data),
                    ] {
                        let color = if active { tc.foreground } else { tc.muted_foreground };
                        let btn = egui::Button::new(RichText::new(label).size(12.5).color(color))
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(0.0, 28.0));
                        let resp = ui.add(btn);
                        if active {
                            // Accent underline
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(resp.rect.left(), resp.rect.bottom()),
                                    egui::vec2(resp.rect.width(), 2.0),
                                ),
                                egui::CornerRadius::ZERO,
                                tc.primary,
                            );
                        }
                        if resp.clicked() {
                            new_view = Some(view);
                        }
                        ui.add_space(4.0);
                    }
                });
            });

            // Border below toolbar
            ui.painter().hline(
                toolbar_rect.x_range(),
                toolbar_rect.bottom(),
                egui::Stroke::new(1.0, tc.border),
            );

            if let Some(v) = new_view {
                if let Some(t) = explorer.tabs.active_tab_mut() {
                    t.view = v;
                }
            }
        }
    }

    // Data grid
    let row_height = app.settings.density.row_height();
    let available = ui.available_rect_before_wrap();
    let status_height = STATUS_H;
    let grid_height = (available.height() - status_height).max(10.0);
    let grid_rect =
        egui::Rect::from_min_size(available.min, egui::vec2(available.width(), grid_height));
    let status_rect = egui::Rect::from_min_size(
        available.min + egui::vec2(0.0, grid_height),
        egui::vec2(available.width(), status_height),
    );

    // Read tab state for rendering
    let (is_loading, has_result, _tab_id_for_load) = {
        let explorer = app.explorer.as_ref().unwrap();
        if let Some(tab) = explorer.tabs.active_tab() {
            (tab.loading, tab.result.is_some(), tab.id.clone())
        } else {
            return;
        }
    };

    {
        let mut grid_ui = ui.new_child(egui::UiBuilder::new().max_rect(grid_rect));
        if is_loading {
            grid_ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Loading…").color(ui.visuals().weak_text_color()));
            });
        } else if has_result {
            let (columns, rows) = {
                let explorer = app.explorer.as_ref().unwrap();
                let tab = explorer.tabs.active_tab().unwrap();
                let result = tab.result.as_ref().unwrap();
                (result.columns.clone(), result.rows.clone())
            };

            egui::ScrollArea::both().show(&mut grid_ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::auto().at_least(36.0))
                    .columns(
                        egui_extras::Column::auto().at_least(80.0).resizable(true),
                        columns.len(),
                    )
                    .header(row_height, |mut header| {
                        header.col(|ui| {
                            ui.label(RichText::new("#").size(11.0).weak());
                        });
                        for col in &columns {
                            header.col(|ui| {
                                ui.horizontal(|ui| {
                                    if col.is_pk {
                                        ui.label(RichText::new("🔑").size(10.0));
                                    }
                                    ui.label(RichText::new(&col.name).size(12.0).strong());
                                    ui.label(RichText::new(&col.data_type).size(10.0).weak());
                                });
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(row_height, rows.len(), |mut row| {
                            let ri = row.index();
                            row.col(|ui| {
                                ui.label(RichText::new((ri + 1).to_string()).size(11.0).weak());
                            });
                            for (ci, col) in columns.iter().enumerate() {
                                row.col(|ui| {
                                    render_cell(ui, col, &rows[ri].get(ci).cloned().flatten());
                                });
                            }
                        });
                    });
            });
        }
    }

    // Status bar + pagination
    let mut status_ui = ui.new_child(egui::UiBuilder::new().max_rect(status_rect));
    status_ui.separator();
    let load_req: Option<LoadRequest> = {
        let explorer = app.explorer.as_ref().unwrap();
        if let Some(tab) = explorer.tabs.active_tab() {
            let tab_snap = tab.clone();
            let (prev, next) = status_bar(&mut status_ui, &tab_snap);
            if (prev && tab.can_go_prev()) || (next && tab.can_go_next()) {
                Some(LoadRequest {
                    tab_id: tab.id.clone(),
                    db: tab.database.clone(),
                    schema: tab.schema.clone(),
                    table: tab.table.clone(),
                    limit: tab.page_size,
                    offset: if prev {
                        (tab.page - 1) * tab.page_size
                    } else {
                        (tab.page + 1) * tab.page_size
                    },
                })
            } else {
                None
            }
        } else {
            None
        }
    };
    if let Some(req) = load_req {
        let explorer = app.explorer.as_mut().unwrap();
        if let Some(tab) = explorer.tabs.active_tab_mut() {
            let going_prev = req.offset < tab.offset();
            if going_prev {
                tab.page -= 1;
            } else {
                tab.page += 1;
            }
            tab.loading = true;
            tab.result = None;
        }
        app.load_rows(
            ctx.clone(),
            req.tab_id,
            req.db,
            req.schema,
            req.table,
            req.limit,
            req.offset,
        );
    }
}
