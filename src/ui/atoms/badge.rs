use crate::core::connections::model::DbEngine;
use egui::{Color32, Ui};

pub fn engine_badge(ui: &mut Ui, engine: DbEngine) {
    let (text, bg) = match engine {
        DbEngine::Postgres => ("PG", Color32::from_rgb(0x3a, 0x6e, 0xa5)),
        DbEngine::MySQL => ("My", Color32::from_rgb(0xc0, 0x82, 0x0f)),
    };
    badge_pill(ui, text, bg, Color32::WHITE);
}

pub fn view_pill(ui: &mut Ui) {
    use crate::theme::colors;
    badge_pill(ui, "VIEW", colors::PURPLE, Color32::WHITE);
}

pub fn count_pill(ui: &mut Ui, count: &str) {
    let (bg, fg) = if ui.visuals().dark_mode {
        (
            Color32::from_rgb(0x22, 0x2a, 0x36),
            Color32::from_rgb(0x99, 0xa3, 0xb2),
        )
    } else {
        (
            Color32::from_rgb(0xe6, 0xe9, 0xee),
            Color32::from_rgb(0x59, 0x62, 0x6f),
        )
    };
    badge_pill(ui, count, bg, fg);
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
