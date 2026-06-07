use crate::core::connections::model::ConnColor;
use egui::{Color32, Ui, Vec2};

pub fn label_dot(ui: &mut Ui, color: ConnColor, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .circle_filled(rect.center(), size / 2.0, color.to_color32());
    }
}

pub fn colored_dot(ui: &mut Ui, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().circle_filled(rect.center(), size / 2.0, color);
    }
}
