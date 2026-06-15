use crate::theme::ThemeColors;
use crate::ui::atoms::button::{primary_button, secondary_button};
use egui::{RichText, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitChoice {
    Pending,
    Confirm,
    Cancel,
}

/// Modal that previews the pending SQL statements (the "diff") before they are
/// committed to the database. Returns the user's choice for this frame.
pub fn commit_dialog(ui: &mut Ui, id: &str, statements: &[String]) -> CommitChoice {
    let tc = ThemeColors::from_ui(ui);
    let mut choice = CommitChoice::Pending;

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
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_min_width(440.0);
                    ui.set_max_width(620.0);

                    ui.label(
                        RichText::new("Review pending changes")
                            .size(15.0)
                            .color(tc.text_primary)
                            .family(egui::FontFamily::Name("SemiBold".into())),
                    );
                    ui.add_space(4.0);
                    let summary = if statements.len() == 1 {
                        "1 statement will run in a single transaction.".to_string()
                    } else {
                        format!(
                            "{} statements will run in a single transaction.",
                            statements.len()
                        )
                    };
                    ui.label(RichText::new(summary).size(12.0).color(tc.text_secondary));

                    ui.add_space(12.0);

                    egui::Frame::new()
                        .fill(tc.background)
                        .stroke(egui::Stroke::new(1.0, tc.border))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(280.0)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    for stmt in statements {
                                        ui.label(
                                            RichText::new(format!("{stmt};"))
                                                .size(12.0)
                                                .font(egui::FontId::monospace(12.0))
                                                .color(tc.text_primary),
                                        );
                                        ui.add_space(4.0);
                                    }
                                });
                        });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if primary_button(ui, "Commit").clicked() {
                                choice = CommitChoice::Confirm;
                            }
                            ui.add_space(8.0);
                            if secondary_button(ui, "Cancel").clicked() {
                                choice = CommitChoice::Cancel;
                            }
                        });
                    });
                });
        });

    choice
}
