use egui::{Ui, Vec2};

/// Renders a two-pane horizontal split.
/// `left_width`: fixed width of the left panel.
/// Returns (left_ui_fn_result, right_ui_fn_result) via closures.
pub fn two_pane<L, R, LR, RR>(ui: &mut Ui, left_width: f32, left_fn: L, right_fn: R)
where
    L: FnOnce(&mut Ui) -> LR,
    R: FnOnce(&mut Ui) -> RR,
{
    let available = ui.available_rect_before_wrap();
    let left_rect = egui::Rect::from_min_size(
        available.min,
        Vec2::new(left_width, available.height()),
    );
    let right_rect = egui::Rect::from_min_size(
        available.min + Vec2::new(left_width + 1.0, 0.0),
        Vec2::new(available.width() - left_width - 1.0, available.height()),
    );

    // Divider
    ui.painter().line_segment(
        [left_rect.right_top(), left_rect.right_bottom()],
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
    );

    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_fn(&mut left_ui);

    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_fn(&mut right_ui);
}
