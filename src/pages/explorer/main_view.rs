use crate::pages::explorer::state::TabView;
use crate::theme::ThemeColors;
use crate::ui::molecules::{
    data_cell::render_cell,
    status_bar::{status_bar, STATUS_H},
    tab_bar::tab_bar,
};
use egui::RichText;

pub fn render_main(ui: &mut egui::Ui, app: &mut crate::app::YssvApp, ctx: &egui::Context) {
    if app.explorer.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Select a connection to explore").size(14.0));
        });
        return;
    }

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

    let row_height = app.settings.density.row_height();
    let available = ui.available_rect_before_wrap();
    let grid_height = (available.height() - STATUS_H).max(10.0);
    let grid_rect =
        egui::Rect::from_min_size(available.min, egui::vec2(available.width(), grid_height));
    let status_rect = egui::Rect::from_min_size(
        available.min + egui::vec2(0.0, grid_height),
        egui::vec2(available.width(), STATUS_H),
    );

    let (is_loading, has_result) = {
        let explorer = app.explorer.as_ref().unwrap();
        if let Some(tab) = explorer.tabs.active_tab() {
            (tab.loading, tab.result.is_some())
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

    let mut status_ui = ui.new_child(egui::UiBuilder::new().max_rect(status_rect));
    status_ui.separator();
    let load_req: Option<super::LoadRequest> = {
        let explorer = app.explorer.as_ref().unwrap();
        if let Some(tab) = explorer.tabs.active_tab() {
            let tab_snap = tab.clone();
            let (prev, next) = status_bar(&mut status_ui, &tab_snap);
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
