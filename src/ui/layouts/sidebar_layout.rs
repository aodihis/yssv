use egui::{Ui, Vec2};

pub fn sidebar_layout<S, M>(ui: &mut Ui, sidebar_width: f32, sidebar_fn: S, main_fn: M)
where
    S: FnOnce(&mut Ui),
    M: FnOnce(&mut Ui),
{
    let available = ui.available_rect_before_wrap();
    let sidebar_rect = egui::Rect::from_min_size(
        available.min,
        Vec2::new(sidebar_width, available.height()),
    );
    let main_rect = egui::Rect::from_min_size(
        available.min + Vec2::new(sidebar_width + 1.0, 0.0),
        Vec2::new(available.width() - sidebar_width - 1.0, available.height()),
    );

    // Divider
    ui.painter().line_segment(
        [sidebar_rect.right_top(), sidebar_rect.right_bottom()],
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
    );

    let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(sidebar_rect));
    sidebar_fn(&mut sidebar_ui);

    let mut main_ui = ui.new_child(egui::UiBuilder::new().max_rect(main_rect));
    main_fn(&mut main_ui);
}
