use crate::app::{Screen, YssvApp};
use crate::theme::{self, colors, ThemeColors};

pub fn render(ctx: &egui::Context, app: &mut YssvApp) {
    let titlebar_bg = if app.settings.theme == theme::Theme::Dark {
        colors::dark::TITLEBAR
    } else {
        colors::light::TITLEBAR
    };

    egui::TopBottomPanel::top("title_bar")
        .exact_size(30.0)
        .frame(
            egui::Frame::new()
                .fill(titlebar_bg)
                .inner_margin(egui::Margin::symmetric(12, 0)),
        )
        .show(ctx, |ui| {
            let tc = ThemeColors::from_ui(ui);
            ui.horizontal_centered(|ui| {
                ui.label(
                    egui::RichText::new("YSSV")
                        .size(13.0)
                        .strong()
                        .color(tc.foreground),
                );

                if matches!(app.screen, Screen::Explorer) {
                    ui.add_space(4.0);
                    let back_btn = egui::Button::new(
                        egui::RichText::new("◀  Connections")
                            .size(12.0)
                            .color(tc.muted_foreground),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, tc.input))
                    .min_size(egui::vec2(0.0, 26.0));
                    if ui.add(back_btn).clicked() {
                        app.screen = Screen::Connections;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        egui::RichText::new("❌").size(12.0).color(tc.muted_foreground),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(30.0, 28.0));

                    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    let maximize_restore_icon = if is_maximized { "🗗" } else { "⬜" };
                    let maximize_restore_btn = egui::Button::new(
                        egui::RichText::new(maximize_restore_icon)
                            .size(12.0)
                            .color(tc.muted_foreground),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(30.0, 28.0));

                    let minimize_btn = egui::Button::new(
                        egui::RichText::new("—").size(12.0).color(tc.muted_foreground),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(30.0, 28.0));

                    let theme_icon = if app.settings.theme == theme::Theme::Dark {
                        "☀"
                    } else {
                        "🌙"
                    };
                    let theme_btn = egui::Button::new(
                        egui::RichText::new(theme_icon)
                            .size(14.0)
                            .color(tc.muted_foreground),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(30.0, 28.0));

                    if ui.add(close_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.add(maximize_restore_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }
                    if ui.add(minimize_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    if ui.add(theme_btn).clicked() {
                        app.settings.toggle_theme();
                    }
                });
            });

            if ui.rect_contains_pointer(ui.max_rect()) {
                if ui.input(|i| i.pointer.primary_pressed()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            }
        });
}
