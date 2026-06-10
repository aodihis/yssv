use crate::theme::ThemeColors;
use egui::*;

pub fn light_switch(ui: &mut Ui, on: &mut bool) -> Response {
    let desired_size = vec2(34.0, 20.0);

    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let tc = ThemeColors::from_ui(ui);

        let bg_color = if *on {
            tc.button_primary_bg
        } else {
            tc.control_track
        };

        ui.painter().rect(
            rect,
            rect.height() / 2.0,
            bg_color,
            Stroke::NONE,
            StrokeKind::Middle,
        );

        let gap = 1.5;
        let radius = rect.height() * 0.45 - gap;
        let circle_x = lerp((rect.left() + radius + gap)..=(rect.right() - radius - gap), how_on);

        ui.painter()
            .circle_filled(pos2(circle_x, rect.center().y), radius, Color32::WHITE);
    }

    response
}
