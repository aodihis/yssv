use egui::{Color32, FontId, Ui};
use crate::pages::explorer::state::TableTab;
use crate::theme::ThemeColors;

pub const STATUS_H: f32 = 34.0;

/// Returns (prev_clicked, next_clicked)
pub fn status_bar(ui: &mut Ui, tab: &TableTab) -> (bool, bool) {
    let tc = ThemeColors::from_ui(ui);
    let mut prev = false;
    let mut next = false;

    let bar_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), STATUS_H),
    );

    // Background + top border
    ui.painter().rect_filled(bar_rect, egui::CornerRadius::ZERO, tc.bg_panel);
    ui.painter().hline(bar_rect.x_range(), bar_rect.top(), egui::Stroke::new(1.0, tc.border));

    ui.scope_builder(egui::UiBuilder::new().max_rect(bar_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(12.0);

            if let Some(result) = &tab.result {
                let total = result.total_rows.unwrap_or(0);
                let from  = tab.offset() + 1;
                let to    = (tab.offset() + tab.page_size).min(total as u32);
                let pages = tab.total_pages();
                let font  = FontId::proportional(11.5);

                // Row count — painter.text returns the bounding Rect; reuse width to advance cursor
                let text_rect = ui.painter().text(
                    ui.cursor().min + egui::vec2(0.0, STATUS_H / 2.0),
                    egui::Align2::LEFT_CENTER,
                    format!("Rows {from}–{to} of {total}"),
                    font,
                    tc.text_muted,
                );
                ui.add_space(text_rect.width() + 16.0);

                // Spacer
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);

                    // Next button
                    let next_resp = pg_button(ui, "▶", tab.can_go_next(), &tc);
                    if next_resp.clicked() { next = true; }

                    // Page indicator
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!("{} / {}", tab.page + 1, pages))
                            .size(11.5)
                            .color(tc.text_muted),
                    );
                    ui.add_space(6.0);

                    // Prev button
                    let prev_resp = pg_button(ui, "◀", tab.can_go_prev(), &tc);
                    if prev_resp.clicked() { prev = true; }
                });
            } else if tab.loading {
                ui.label(
                    egui::RichText::new("Loading…")
                        .size(11.5)
                        .color(tc.text_faint),
                );
            }
        });
    });

    (prev, next)
}

fn pg_button(ui: &mut Ui, label: &str, enabled: bool, tc: &ThemeColors) -> egui::Response {
    let size = egui::vec2(24.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if !enabled {
            Color32::from_rgba_unmultiplied(tc.text_faint.r(), tc.text_faint.g(), tc.text_faint.b(), 80)
        } else if resp.hovered() {
            let painter = ui.painter();
            painter.rect_filled(rect, egui::CornerRadius::same(5u8), tc.bg_hover);
            tc.text
        } else {
            tc.text_muted
        };
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, FontId::proportional(11.0), color);
    }
    if enabled { resp } else { resp.with_new_rect(rect) }
}
