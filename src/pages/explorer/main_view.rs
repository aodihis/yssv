use crate::core::results::model::ColumnDef;
use crate::pages::explorer::state::{Tab, TabView};
use crate::theme::ThemeColors;
use crate::ui::atoms::button::compact_button;
use crate::ui::atoms::sql_highlight::sql_area;
use crate::ui::molecules::{
    data_table::data_table,
    status_bar::{STATUS_H, status_bar},
    tab_bar::tab_bar,
};
use egui::RichText;
use crate::ui::atoms::dropdown::dropdown;

const EDITOR_H: f32 = 220.0;
const QUERY_STATUS_H: f32 = 28.0;

pub fn render_main(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let ctx = ui.ctx().clone();
    if app.explorer.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a connection to explore").size(14.0));
        });
        return;
    }

    let (activate, close) = {
        let explorer = app
            .explorer
            .as_ref()
            .expect("explorer is Some — checked above");
        if explorer.tabs.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select a table or open a SQL query")
                        .size(14.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });
            return;
        }
        tab_bar(ui, &explorer.tabs.tabs, explorer.tabs.active)
    };
    {
        let explorer = app
            .explorer
            .as_mut()
            .expect("explorer is Some — checked above");
        if let Some(i) = activate {
            explorer.tabs.active = i;
        }
        if let Some(i) = close {
            explorer.tabs.close(i);
        }
        if explorer.tabs.tabs.is_empty() {
            return;
        }
    }

    // Decide which kind of tab is active
    let is_query = app
        .explorer
        .as_ref()
        .and_then(|e| e.tabs.active_tab())
        .map(|t| matches!(t, Tab::Query(_)))
        .unwrap_or(false);

    if is_query {
        render_query_tab(ui, app, &ctx);
    } else {
        render_table_tab(ui, app, &ctx);
    }
}

// ---------------------------------------------------------------------------
// Query tab
// ---------------------------------------------------------------------------

fn render_query_tab(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    let tc = ThemeColors::from_ui(ui);

    ui.add_space(8.0);

    // Snapshot query tab state + available databases
    let query_snapshot = app
        .explorer
        .as_ref()
        .and_then(|e| {
            e.tabs.active_query_tab().map(|q| {
                let db_names: Vec<String> = e.databases.iter().map(|d| d.name.clone()).collect();
                let multi_db = db_names.len() > 1;
                (
                    q.id.clone(),
                    q.database.clone(),
                    q.loading,
                    q.result.is_some(),
                    q.error.clone(),
                    q.selected_row,
                    db_names,
                    multi_db,
                )
            })
        });

    let (tab_id, database, loading, has_result, error, selected_row, db_names, multi_db) =
        match query_snapshot {
            Some(s) => s,
            None => return,
        };

    // Toolbar: Run button + database indicator
    let toolbar_rect =
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 40.0));
    ui.painter()
        .rect_filled(toolbar_rect, egui::CornerRadius::ZERO, tc.surface);

    let mut run_clicked = false;
    let mut selected_db = database.clone();
    ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(12.0);
            if compact_button(ui, "▶ Run").clicked() {
                run_clicked = true;
            }
            ui.add_space(12.0);
            if multi_db {
                ui.label(
                    RichText::new("db:")
                        .size(11.0)
                        .color(tc.text_disabled),
                );
                ui.add_space(4.0);
                let db_options: Vec<(String, &str)> =
                    db_names.iter().map(|n| (n.clone(), n.as_str())).collect();
                ui.allocate_ui(egui::vec2(160.0, ui.available_height()), |ui| {
                    dropdown(ui, "query_db_selector", &mut selected_db, &db_options, 200.0);
                });
            } else {
                ui.label(
                    RichText::new(format!("db: {database}"))
                        .size(11.0)
                        .color(tc.text_disabled),
                );
            }
        });
    });
    // Persist database selection change back to the tab
    if selected_db != database {
        if let Some(q) = app.explorer.as_mut().and_then(|e| e.tabs.active_query_tab_mut()) {
            q.database = selected_db.clone();
        }
    }
    ui.painter().hline(
        toolbar_rect.x_range(),
        toolbar_rect.bottom(),
        egui::Stroke::new(1.0, tc.border),
    );

    let available = ui.available_rect_before_wrap();
    let editor_h = EDITOR_H.min(available.height() * 0.45);
    let status_h = QUERY_STATUS_H;
    const RESULTS_GAP: f32 = 8.0;
    let results_h = (available.height() - editor_h - status_h - 1.0 - RESULTS_GAP).max(40.0);

    let editor_rect =
        egui::Rect::from_min_size(available.min, egui::vec2(available.width(), editor_h));
    let results_rect = egui::Rect::from_min_size(
        available.min + egui::vec2(0.0, editor_h + 1.0 + RESULTS_GAP),
        egui::vec2(available.width(), results_h),
    );
    let status_rect = egui::Rect::from_min_size(
        available.min + egui::vec2(0.0, editor_h + 1.0 + RESULTS_GAP + results_h),
        egui::vec2(available.width(), status_h),
    );

    // SQL editor — mutable borrow of tab.sql
    let ctrl_enter = {
        let mut triggered = false;
        if let Some(q) = app.explorer.as_mut().and_then(|e| e.tabs.active_query_tab_mut()) {
            let mut editor_ui = ui.new_child(egui::UiBuilder::new().max_rect(editor_rect));
            let resp = sql_area(&mut editor_ui, &mut q.sql, "SELECT * FROM table…", 8);
            if resp.has_focus()
                && editor_ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
            {
                triggered = true;
            }
        }
        triggered
    };

    // Separator between editor and results
    ui.painter().hline(
        egui::Rangef::new(available.left(), available.right()),
        available.min.y + editor_h,
        egui::Stroke::new(1.0, tc.border),
    );

    // Trigger query execution
    if (run_clicked || ctrl_enter) && !loading {
        let sql = app
            .explorer
            .as_ref()
            .and_then(|e| e.tabs.active_query_tab())
            .map(|q| q.sql.trim().to_string())
            .unwrap_or_default();

        if !sql.is_empty() {
            if let Some(q) = app.explorer.as_mut().and_then(|e| e.tabs.active_query_tab_mut()) {
                q.loading = true;
                q.result = None;
                q.error = None;
            }
            // Use selected_db (possibly just changed this frame) as the target database
            app.run_query(ctx.clone(), tab_id.clone(), sql, selected_db.clone());
        }
    }

    // Results area
    let mut results_ui = ui.new_child(egui::UiBuilder::new().max_rect(results_rect));
    let row_height = app.settings.density.row_height();

    if loading {
        results_ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Running…").color(ui.visuals().weak_text_color()));
        });
    } else if let Some(ref err) = error {
        results_ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.add_space(12.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::same(12))
                .corner_radius(egui::CornerRadius::same(6u8))
                .fill(egui::Color32::from_rgba_unmultiplied(
                    tc.error.r(),
                    tc.error.g(),
                    tc.error.b(),
                    20,
                ))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(err.as_str())
                            .size(12.0)
                            .color(tc.error)
                            .font(egui::FontId::monospace(12.0)),
                    );
                });
        });
    } else if has_result {
        let mut new_selected = selected_row;

        if let Some(explorer) = app.explorer.as_ref()
            && let Some(q) = explorer.tabs.active_query_tab()
            && let Some(result) = q.result.as_ref()
        {
            let columns = &result.columns;
            let rows = &result.rows;

            if rows.is_empty() {
                results_ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("No rows returned")
                            .size(13.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                });
            } else {
                new_selected = data_table(&mut results_ui, columns, rows, row_height, new_selected);
            }
        }

        if new_selected != selected_row
            && let Some(q) = app.explorer.as_mut().and_then(|e| e.tabs.active_query_tab_mut())
        {
            q.selected_row = new_selected;
        }
    } else {
        results_ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Press ▶ Run or Ctrl+Enter to execute the query")
                    .size(13.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });
    }

    // Query status strip
    let mut status_ui = ui.new_child(egui::UiBuilder::new().max_rect(status_rect));
    status_ui.painter().hline(
        status_rect.x_range(),
        status_rect.top(),
        egui::Stroke::new(1.0, tc.border),
    );
    status_ui.painter().rect_filled(status_rect, egui::CornerRadius::ZERO, tc.surface);

    let status_text = if loading {
        "Running…".to_string()
    } else if error.is_some() {
        "Error".to_string()
    } else if let Some(explorer) = app.explorer.as_ref()
        && let Some(q) = explorer.tabs.active_query_tab()
        && let Some(result) = q.result.as_ref()
    {
        format!("{} rows", result.rows.len())
    } else {
        String::new()
    };

    if !status_text.is_empty() {
        status_ui.scope_builder(egui::UiBuilder::new().max_rect(status_rect), |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new(&status_text)
                        .size(11.5)
                        .color(if error.is_some() { tc.error } else { tc.text_secondary }),
                );
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Structure table renderer (shared)
// ---------------------------------------------------------------------------

fn render_structure_table(ui: &mut egui::Ui, columns: &[ColumnDef], row_height: f32) {
    let tc = ThemeColors::from_ui(ui);
    egui::ScrollArea::both().show(ui, |ui| {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto().at_least(36.0))
            .column(egui_extras::Column::auto().at_least(140.0).resizable(true))
            .column(egui_extras::Column::auto().at_least(120.0).resizable(true))
            .column(egui_extras::Column::auto().at_least(70.0))
            .column(egui_extras::Column::auto().at_least(40.0))
            .column(egui_extras::Column::auto().at_least(40.0))
            .header(row_height, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("#").size(11.0).weak());
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Column")
                            .size(12.0)
                            .family(egui::FontFamily::Name("SemiBold".into())),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Type")
                            .size(12.0)
                            .family(egui::FontFamily::Name("SemiBold".into())),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Nullable")
                            .size(12.0)
                            .family(egui::FontFamily::Name("SemiBold".into())),
                    );
                });
                header.col(|ui| {
                    ui.label(RichText::new("PK").size(12.0).weak());
                });
                header.col(|ui| {
                    ui.label(RichText::new("FK").size(12.0).weak());
                });
            })
            .body(|body| {
                body.rows(row_height, columns.len(), |mut row| {
                    let i = row.index();
                    let col = &columns[i];
                    row.col(|ui| {
                        ui.label(RichText::new((i + 1).to_string()).size(11.0).weak());
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(&col.name)
                                .size(12.0)
                                .family(egui::FontFamily::Name("SemiBold".into())),
                        );
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(&col.data_type)
                                .size(12.0)
                                .font(egui::FontId::monospace(12.0))
                                .color(tc.text_secondary),
                        );
                    });
                    row.col(|ui| {
                        let (text, color) = if col.nullable {
                            ("YES", tc.text_secondary)
                        } else {
                            ("NO", tc.text_primary)
                        };
                        ui.label(RichText::new(text).size(11.0).color(color));
                    });
                    row.col(|ui| {
                        if col.is_pk {
                            ui.label(RichText::new("🔑").size(11.0));
                        }
                    });
                    row.col(|ui| {
                        if col.is_fk {
                            ui.label(RichText::new("🔗").size(11.0));
                        }
                    });
                });
            });
    });
}

// ---------------------------------------------------------------------------
// Table tab
// ---------------------------------------------------------------------------

fn render_table_tab(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    let tc = ThemeColors::from_ui(ui);
    let toolbar_rect =
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 32.0));
    ui.painter()
        .rect_filled(toolbar_rect, egui::CornerRadius::ZERO, tc.surface);

    let explorer = app
        .explorer
        .as_mut()
        .expect("explorer is Some — checked above");
    if let Some(tab) = explorer.tabs.active_table_tab() {
        let is_data = tab.view == TabView::Data;
        let mut new_view: Option<TabView> = None;
        ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar_rect), |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(12.0);
                for (label, view, active) in [
                    ("Data", TabView::Data, is_data),
                    ("Structure", TabView::Structure, !is_data),
                ] {
                    let color = if active {
                        tc.text_primary
                    } else {
                        tc.text_secondary
                    };
                    let btn = egui::Button::new(RichText::new(label).size(12.5).color(color))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .min_size(egui::vec2(0.0, 28.0));
                    let resp = ui.add(btn);
                    if active {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(resp.rect.left(), resp.rect.bottom()),
                                egui::vec2(resp.rect.width(), 2.0),
                            ),
                            egui::CornerRadius::ZERO,
                            tc.button_primary_bg,
                        );
                    }
                    if resp.clicked() {
                        new_view = Some(view);
                    }
                    ui.add_space(4.0);
                }
            });
        });

        ui.painter().hline(
            toolbar_rect.x_range(),
            toolbar_rect.bottom(),
            egui::Stroke::new(1.0, tc.border),
        );

        if let Some(v) = new_view
            && let Some(t) = explorer.tabs.active_table_tab_mut()
        {
            t.view = v;
        }
    }

    // Trigger structure load if needed
    let structure_load: Option<(String, String, String, String)> = {
        let explorer = app
            .explorer
            .as_ref()
            .expect("explorer is Some — checked above");
        if let Some(tab) = explorer.tabs.active_table_tab() {
            if tab.view == TabView::Structure && tab.structure.is_none() && !tab.structure_loading {
                Some((
                    tab.id.clone(),
                    tab.database.clone(),
                    tab.schema.clone(),
                    tab.table.clone(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    };
    if let Some((tab_id, db, schema, table)) = structure_load {
        if let Some(tab) = app
            .explorer
            .as_mut()
            .expect("explorer is Some — checked above")
            .tabs
            .active_table_tab_mut()
        {
            tab.structure_loading = true;
        }
        app.load_structure(ctx.clone(), tab_id, db, schema, table);
    }

    let row_height = app.settings.density.row_height();
    let available = ui.available_rect_before_wrap();
    let grid_height = (available.height() - STATUS_H).max(10.0);
    let grid_rect =
        egui::Rect::from_min_size(available.min, egui::vec2(available.width(), grid_height));
    let status_rect = egui::Rect::from_min_size(
        available.min + egui::vec2(0.0, grid_height),
        egui::vec2(available.width(), STATUS_H),
    );

    let tab_info = {
        let explorer = app
            .explorer
            .as_ref()
            .expect("explorer is Some — checked above");
        explorer.tabs.active_table_tab().map(|tab| {
            (
                tab.view,
                tab.loading,
                tab.result.is_some(),
                tab.structure_loading,
                tab.structure.is_some(),
                tab.selected_row,
            )
        })
    };
    let (tab_view, is_loading, has_result, struct_loading, has_structure, selected_row) =
        match tab_info {
            Some(t) => t,
            None => return,
        };

    let mut grid_ui = ui.new_child(egui::UiBuilder::new().max_rect(grid_rect));

    match tab_view {
        TabView::Structure => {
            if struct_loading {
                grid_ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Loading structure…").color(ui.visuals().weak_text_color()),
                    );
                });
            } else if has_structure
                && let Some(explorer) = app.explorer.as_ref()
                && let Some(tab) = explorer.tabs.active_table_tab()
                && let Some(cols) = tab.structure.as_deref()
            {
                render_structure_table(&mut grid_ui, cols, row_height);
            }
        }
        TabView::Data => {
            if is_loading {
                grid_ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Loading…").color(ui.visuals().weak_text_color()));
                });
            } else if has_result {
                let mut new_selected = selected_row;

                if let Some(explorer) = app.explorer.as_ref()
                    && let Some(tab) = explorer.tabs.active_table_tab()
                    && let Some(result) = tab.result.as_ref()
                {
                    let columns = &result.columns;
                    let rows = &result.rows;

                    new_selected = data_table(&mut grid_ui, columns, rows, row_height, new_selected);

                    if let Some(sel_ri) = new_selected
                        && grid_ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C))
                    {
                        let text = rows[sel_ri]
                            .iter()
                            .map(|v| v.as_deref().unwrap_or_default())
                            .collect::<Vec<_>>()
                            .join("\t");
                        grid_ui.ctx().copy_text(text);
                    }
                }

                if new_selected != selected_row
                    && let Some(tab) = app
                        .explorer
                        .as_mut()
                        .and_then(|e| e.tabs.active_table_tab_mut())
                {
                    tab.selected_row = new_selected;
                }
            }
        }
    }

    let mut status_ui = ui.new_child(egui::UiBuilder::new().max_rect(status_rect));
    status_ui.separator();
    let load_req: Option<super::LoadRequest> = {
        let explorer = app
            .explorer
            .as_ref()
            .expect("explorer is Some — checked above");
        if let Some(tab) = explorer.tabs.active_table_tab()
            && tab.view == TabView::Data
        {
            let (prev, next) = status_bar(&mut status_ui, tab);
            if (prev && tab.can_go_prev()) || (next && tab.can_go_next()) {
                Some(super::LoadRequest {
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
        let explorer = app
            .explorer
            .as_mut()
            .expect("explorer is Some — checked above");
        if let Some(tab) = explorer.tabs.active_table_tab_mut() {
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
