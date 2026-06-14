use crate::theme::ThemeColors;
use crate::ui::atoms::icon::{Icon, icon_image};
use egui::{RichText, Ui};

pub fn error_dialog(ui: &mut Ui, id: &str, title: &str, message: &str) -> bool {
    let tc = ThemeColors::from_ui(ui);
    let mut dismissed = false;

    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new(id).with("scrim"),
        ))
        .rect_filled(
            ui.ctx().content_rect(),
            egui::CornerRadius::ZERO,
            egui::Color32::from_black_alpha(140),
        );

    egui::Area::new(egui::Id::new(id).with("dialog"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(tc.surface)
                .stroke(egui::Stroke::new(1.0, tc.border))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_min_width(340.0);
                    ui.set_max_width(480.0);

                    ui.vertical_centered(|ui| {
                        let (rect, _) = ui
                            .allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            rect.center(),
                            22.0,
                            egui::Color32::from_rgba_unmultiplied(
                                tc.error.r(),
                                tc.error.g(),
                                tc.error.b(),
                                30,
                            ),
                        );
                        ui.painter().circle_stroke(
                            rect.center(),
                            22.0,
                            egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(
                                    tc.error.r(),
                                    tc.error.g(),
                                    tc.error.b(),
                                    60,
                                ),
                            ),
                        );
                        let icon_rect = egui::Rect::from_center_size(
                            rect.center(),
                            egui::vec2(20.0, 20.0),
                        );
                        ui.put(icon_rect, icon_image(Icon::X, 20.0, tc.error));
                    });

                    ui.add_space(16.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(title)
                                .size(15.0)
                                .color(tc.text_primary)
                                .family(egui::FontFamily::Name("SemiBold".into())),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(message)
                                .size(12.5)
                                .color(tc.text_secondary),
                        );
                    });

                    ui.add_space(20.0);

                    let div_rect = egui::Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(ui.available_width(), 1.0),
                    );
                    ui.painter()
                        .rect_filled(div_rect, egui::CornerRadius::ZERO, tc.border);
                    ui.add_space(1.0);
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let btn = egui::Button::new(
                                    RichText::new("Dismiss").size(13.0).color(tc.text_primary),
                                )
                                .fill(tc.background)
                                .stroke(egui::Stroke::new(1.0, tc.field_border))
                                .corner_radius(6.0)
                                .min_size(egui::vec2(80.0, 32.0));
                                if ui.add(btn).clicked() {
                                    dismissed = true;
                                }
                            },
                        );
                    });
                });
        });

    dismissed
}
