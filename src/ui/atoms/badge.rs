use crate::core::connections::model::DbEngine;
use crate::theme::{ThemeColors, colors};
use egui::{Color32, Ui};

pub fn engine_badge(ui: &mut Ui, engine: DbEngine) {
    let (text, bg) = match engine {
        DbEngine::Postgres => ("PG", colors::POSTGRES),
        DbEngine::MySQL => ("My", colors::MYSQL),
    };
    badge_pill(ui, text, bg, Color32::WHITE);
}

pub fn view_pill(ui: &mut Ui) {
    badge_pill(ui, "VIEW", colors::PURPLE, Color32::WHITE);
}

pub fn count_pill(ui: &mut Ui, count: &str) {
    let tc = ThemeColors::from_ui(ui);
    badge_pill(ui, count, tc.surface_active, tc.text_secondary);
}

fn badge_pill(ui: &mut Ui, text: &str, bg: Color32, fg: Color32) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), egui::FontId::proportional(10.0), fg);
    let padding = egui::vec2(6.0, 2.0);
    let size = galley.size() + padding * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(255u8), bg);
        ui.painter().galley(rect.min + padding, galley, fg);
    }
}
