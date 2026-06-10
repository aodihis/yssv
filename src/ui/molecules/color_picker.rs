use crate::core::connections::model::ConnColor;
use crate::theme::ThemeColors;
use egui::{Ui, Vec2};

pub fn color_picker(ui: &mut Ui, current: &mut ConnColor) {
    let tc = ThemeColors::from_ui(ui);
    ui.horizontal(|ui| {
        for &color in ConnColor::all() {
            let size = Vec2::splat(20.0);
            let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
            if ui.is_rect_visible(rect) {
                let c32 = color.to_color32();
                ui.painter().circle_filled(rect.center(), 8.0, c32);
                if *current == color {
                    ui.painter().circle_stroke(
                        rect.center(),
                        10.0,
                        egui::Stroke::new(2.0, tc.text_primary),
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
