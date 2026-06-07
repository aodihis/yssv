use crate::core::connections::model::ConnColor;
use egui::{Color32, Ui, Vec2};

pub fn color_picker(ui: &mut Ui, current: &mut ConnColor) {
    ui.horizontal(|ui| {
        for &color in ConnColor::all() {
            let size = Vec2::splat(20.0);
            let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
            if ui.is_rect_visible(rect) {
                let c32 = color.to_color32();
                ui.painter().circle_filled(rect.center(), 8.0, c32);
                if *current == color {
                    // White ring for selected
                    ui.painter().circle_stroke(
                        rect.center(),
                        10.0,
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }
            }
            if resp.clicked() {
                *current = color;
            }
        }
        ui.label(
            egui::RichText::new(current.label())
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    });
}
