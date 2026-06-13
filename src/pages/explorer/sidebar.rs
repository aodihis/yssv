use crate::events::Screen;
use crate::core::schema::model::TableKind;
use crate::theme::{self, ThemeColors, colors};
use crate::ui::atoms::button::compact_button;
use crate::ui::atoms::icon::Icon;
use crate::ui::atoms::input::text_input;
use crate::ui::molecules::tree_row::{TreeRowConfig, tree_row};
use egui::RichText;

type TableRow = (String, TableKind, Option<u64>, bool);
type SchemaRow = (String, bool, usize, Vec<TableRow>);
type DbRow = (String, bool, bool, Vec<SchemaRow>);

pub fn render_sidebar(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let ctx = ui.ctx().clone();
    if app.explorer.is_none() {
        return;
    }
    let accent_color = ui.visuals().selection.stroke.color;
    let tc = ThemeColors::from_ui(ui);

    let conn_name = app
        .explorer
        .as_ref()
        .expect("explorer is Some — checked above")
        .conn_name
        .clone();

    let mut new_query_clicked = false;
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 10,
            bottom: 8,
        })
        .show(ui, |ui| {
            // Header row: connection name + SQL button
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&conn_name)
                        .size(13.0)
                        .family(egui::FontFamily::Name("SemiBold".into()))
                        .color(tc.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if compact_button(ui, "SQL").clicked() {
                        new_query_clicked = true;
                    }
                });
            });
            ui.add_space(7.0);
            let e = app
                .explorer
                .as_mut()
                .expect("explorer is Some — checked above");
            text_input(ui, &mut e.filter, "Filter tables…", Some("🔍"));
        });

    if new_query_clicked {
        let active_db = app
            .explorer
            .as_ref()
            .map(|e| e.active_db.clone())
            .unwrap_or_default();
        if let Some(e) = app.explorer.as_mut() {
            e.tabs.open_query(&active_db);
        }
    }

    let sep = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
    ui.painter()
        .rect_filled(sep, egui::CornerRadius::ZERO, tc.border_muted);
    ui.add_space(1.0);

    let tree_data: Vec<DbRow> = {
        let e = app
            .explorer
            .as_ref()
            .expect("explorer is Some — checked above");
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
                            let filtered_count = if q.is_empty() {
                                sc.tables.len()
                            } else {
                                sc.tables
                                    .iter()
                                    .filter(|t| t.name.to_lowercase().contains(&q))
                                    .count()
                            };
                            let tables = if sc_open {
                                sc.tables
                                    .iter()
                                    .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q))
                                    .map(|t| {
                                        let active = e
                                            .tabs
                                            .active_table_tab()
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
                            (sc.name.clone(), sc_open, filtered_count, tables)
                        })
                        .collect()
                } else {
                    vec![]
                };
                (db.name.clone(), db_open, is_active, schemas)
            })
            .collect()
    };

    let mut toggle_node: Option<String> = None;
    let mut open_table: Option<(String, String, String)> = None;

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
                    label: db_name.clone(),
                    level: 0,
                    is_leaf: false,
                    is_open: *db_open,
                    is_active: *is_active,
                    icon: Icon::Database,
                    icon_color: if *is_active { Some(accent_color) } else { None },
                    count: None,
                    pill: None,
                },
            );
            if resp.clicked() {
                toggle_node = Some(db_key);
            }

            for (schema_name, sc_open, filtered_count, tables) in schemas {
                let sc_key = format!("sc:{}:{}", db_name, schema_name);
                let resp = tree_row(
                    ui,
                    TreeRowConfig {
                        label: schema_name.clone(),
                        level: 1,
                        is_leaf: false,
                        is_open: *sc_open,
                        is_active: false,
                        icon: Icon::Layers,
                        icon_color: None,
                        count: Some(filtered_count.to_string()),
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
                            label: table_name.clone(),
                            level: 2,
                            is_leaf: true,
                            is_open: false,
                            is_active: *active_table,
                            icon: if is_view { Icon::Eye } else { Icon::Table2 },
                            icon_color: if is_view { Some(colors::PURPLE) } else { None },
                            count: if is_view { None } else { count_str },
                            pill: if is_view {
                                Some("VIEW".to_string())
                            } else {
                                None
                            },
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

    let mut load_schemas_for: Option<String> = None;
    if let Some(ref key) = toggle_node
        && let Some(e) = &mut app.explorer
    {
        let was_open = e.is_open(key);
        e.toggle_node(key);
        // Always repaint so the expanded/collapsed state renders on the very next frame,
        // not on the next input event (which could be delayed in reactive mode).
        ctx.request_repaint();
        if !was_open {
            if let Some(stripped) = key.strip_prefix("db:") {
                let db_name = stripped.to_string();
                if let Some(db) = e.databases.iter().find(|d| d.name == db_name)
                    && db.schemas.is_empty()
                {
                    load_schemas_for = Some(db_name);
                }
            } else if key.starts_with("sc:") {
                // fallback: if the parent db never loaded schemas, load now
                let parts: Vec<&str> = key.splitn(3, ':').collect();
                if parts.len() == 3 {
                    let db_name = parts[1].to_string();
                    if let Some(db) = e.databases.iter().find(|d| d.name == db_name)
                        && db.schemas.is_empty()
                    {
                        load_schemas_for = Some(db_name);
                    }
                }
            }
        }
    }
    if let Some(db) = load_schemas_for {
        app.load_schemas(ctx.clone(), db);
    }
    if let Some((table, schema, db)) = open_table {
        let load_req = {
            let e = app
                .explorer
                .as_mut()
                .expect("explorer is Some — open_table came from tree_data which borrows explorer");
            let tab = e.tabs.open_table(&table, &schema, &db);
            if tab.result.is_none() && !tab.loading {
                tab.loading = true;
                Some(super::LoadRequest {
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

    let mut footer_ui = ui.new_child(egui::UiBuilder::new().max_rect(footer_rect));
    footer_ui.painter().hline(
        footer_rect.x_range(),
        footer_rect.top(),
        egui::Stroke::new(1.0, tc.border_muted),
    );
    footer_ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.add_space(1.0);
        let back_btn = egui::Button::new(
            egui::RichText::new("◀  Connections")
                .size(11.0)
                .color(tc.text_secondary),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, tc.border_muted))
        .min_size(egui::vec2(0.0, 22.0));
        if ui.add(back_btn).clicked() {
            app.screen = Screen::Connections;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            let theme_icon = if app.settings.theme == theme::Theme::Dark {
                "☀"
            } else {
                "🌙"
            };
            let theme_btn = egui::Button::new(
                egui::RichText::new(theme_icon)
                    .size(14.0)
                    .color(tc.text_secondary),
            )
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
            .min_size(egui::vec2(28.0, 24.0));
            if ui.add(theme_btn).clicked() {
                app.settings.toggle_theme();
                theme::apply_theme(&ctx, app.settings.theme);
            }
        });
    });
}
