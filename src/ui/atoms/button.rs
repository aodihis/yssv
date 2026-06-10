use crate::theme::ThemeColors;
use egui::{Button, Color32, Response, Ui, Vec2};

/// Full-size primary button (32 px) — for main form CTAs like "Connect" or "Save".
pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let tc = ThemeColors::from_ui(ui);
    ui.add(
        Button::new(egui::RichText::new(label).color(Color32::WHITE))
            .fill(tc.button_primary_bg)
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

/// Compact primary button — matches text_input height for inline use next to inputs.
/// Uses the same vertical rhythm as the input (7 px top/bottom padding, 13 px font).
pub fn compact_button(ui: &mut Ui, label: &str) -> Response {
    let tc = ThemeColors::from_ui(ui);
    ui.scope(|ui| {
        // Mirror the text_input margin (top:7, bottom:7) so heights stay equal.
        // Also cap interact_size.y so egui doesn't inflate the button beyond our padding.
        ui.spacing_mut().button_padding = egui::vec2(15.0, 6.0);
        ui.spacing_mut().interact_size.y = 28.0;
        ui.spacing_mut().interact_size.x = 69.0;
        ui.add(
            Button::new(egui::RichText::new(label).color(Color32::WHITE))
                .fill(tc.button_primary_bg),
        )
    })
    .inner
}
