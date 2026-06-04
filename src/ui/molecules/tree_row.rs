use egui::{Color32, Response, RichText, Ui, Vec2};

pub struct TreeRowConfig<'a> {
    pub label: &'a str,
    pub level: u32,
    pub is_leaf: bool,
    pub is_open: bool,
    pub is_active: bool,
    pub icon: &'a str,
    pub icon_color: Option<Color32>,
    pub count: Option<String>,
    pub pill: Option<&'a str>,
}

pub fn tree_row(ui: &mut Ui, cfg: TreeRowConfig<'_>) -> Response {
    let indent = cfg.level as f32 * 16.0;
    let row_h = 26.0;
    let desired = Vec2::new(ui.available_width(), row_h);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg = if cfg.is_active {
            ui.visuals().selection.bg_fill
        } else if resp.hovered() {
            ui.visuals().faint_bg_color
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, egui::CornerRadius::same(4u8), bg);

        let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        inner.horizontal_centered(|ui| {
            ui.add_space(8.0 + indent);

            // Chevron
            if !cfg.is_leaf {
                let chev = if cfg.is_open { "▾" } else { "▸" };
                ui.label(RichText::new(chev).size(10.0).color(ui.visuals().weak_text_color()));
            } else {
                ui.add_space(14.0);
            }

            // Icon
            let icon_color = cfg.icon_color.unwrap_or(ui.visuals().text_color());
            ui.label(RichText::new(cfg.icon).size(13.0).color(icon_color));
            ui.add_space(4.0);

            // Label
            let label_color = if cfg.is_active {
                ui.visuals().text_color()
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(cfg.label).size(13.0).color(label_color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if let Some(pill) = cfg.pill {
                    let pill_color = crate::theme::colors::PURPLE;
                    ui.label(
                        RichText::new(pill)
                            .size(10.0)
                            .color(pill_color)
                            .background_color(Color32::from_rgba_unmultiplied(
                                pill_color.r(), pill_color.g(), pill_color.b(), 30,
                            )),
                    );
                } else if let Some(count) = &cfg.count {
                    ui.label(
                        RichText::new(count)
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                }
            });
        });
    }

    resp
}
