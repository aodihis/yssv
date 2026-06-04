use egui::{Color32, Response, RichText, Ui, Vec2};
use crate::core::connections::model::Connection;
use crate::ui::atoms::{badge::engine_badge, label_dot::colored_dot};

pub fn conn_item(ui: &mut Ui, conn: &Connection, selected: bool) -> Response {
    let desired_size = Vec2::new(ui.available_width(), 44.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg = if selected {
            ui.visuals().selection.bg_fill
        } else if resp.hovered() {
            ui.visuals().faint_bg_color
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, egui::CornerRadius::same(6u8), bg);

        let mut inner_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        inner_ui.horizontal_centered(|ui| {
            ui.add_space(10.0);
            colored_dot(ui, conn.color.to_color32(), 8.0);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.add_space(6.0);
                ui.label(RichText::new(&conn.name).size(13.0).strong());
                ui.label(
                    RichText::new(conn.display_host())
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                engine_badge(ui, conn.engine);
            });
        });
    }

    resp
}
