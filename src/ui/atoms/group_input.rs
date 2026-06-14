use crate::theme::ThemeColors;
use crate::ui::atoms::input::text_input;
use egui::Ui;

pub fn group_input(ui: &mut Ui, value: &mut String, groups: &[String]) {
    let tc = ThemeColors::from_ui(ui);
    let popup_id = ui.make_persistent_id("group_input_popup");

    let resp = text_input(ui, value, "Local", None);

    if !resp.has_focus() {
        return;
    }

    let q = value.to_lowercase();
    let matching: Vec<&String> = groups
        .iter()
        .filter(|g| q.is_empty() || g.to_lowercase().contains(&q))
        .collect();

    if matching.is_empty() {
        return;
    }

    let field_rect = resp.rect;

    egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(field_rect.min.x, field_rect.max.y + 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::new(1.0, tc.border))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::same(4))
                .show(ui, |ui| {
                    ui.set_min_width(field_rect.width().max(160.0));
                    ui.set_max_width(field_rect.width().max(240.0));
                    for group in &matching {
                        let selected = value.as_str() == group.as_str();
                        let row = ui.add(
                            egui::Button::selectable(
                                selected,
                                egui::RichText::new(*group).size(13.0).color(if selected {
                                    tc.button_primary_bg
                                } else {
                                    tc.text_primary
                                }),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(0.0, 28.0)),
                        );
                        if row.clicked() {
                            *value = group.to_string();
                        }
                    }
                });
        });
}
