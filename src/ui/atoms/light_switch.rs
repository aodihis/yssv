use egui::*;

pub fn light_switch(ui: &mut Ui, on: &mut bool) -> Response {
    let desired_size = vec2(34.0, 20.0);

    let (rect, mut response) = ui.allocate_exact_size(
        desired_size,
        Sense::click(),
    );

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);

        let bg_color = if *on {
            Color32::from_rgb(0, 200, 0)
        } else {
            Color32::from_gray(100)
        };

        ui.painter().rect(
            rect,
            rect.height() / 2.0,
            bg_color,
            Stroke::NONE,
            StrokeKind::Middle,
        );

        let radius = rect.height() * 0.45;
        let circle_x = lerp(
            (rect.left() + radius)..=(rect.right() - radius),
            how_on,
        );

        ui.painter().circle_filled(
            pos2(circle_x, rect.center().y),
            radius,
            Color32::WHITE,
        );
    }

    response
}