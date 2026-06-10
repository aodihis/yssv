use crate::theme::ThemeColors;
use egui::{Response, Stroke, Ui};

/// Generic dropdown (ComboBox).
///
/// `options` is a slice of `(value, label)` pairs. The widget renders the label
/// of the currently selected value and swaps to whichever item the user picks.
pub fn dropdown<T: PartialEq + Clone>(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: &mut T,
    options: &[(T, &str)],
) -> Response {
    let current_label = options
        .iter()
        .find(|(v, _)| v == selected)
        .map(|(_, l)| *l)
        .unwrap_or("—");

    let tc = ThemeColors::from_ui(ui);

    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = tc.background;
        ui.visuals_mut().widgets.hovered.bg_fill = tc.accent;
        ui.visuals_mut().widgets.active.bg_fill = tc.accent_active;
        ui.visuals_mut().widgets.open.bg_fill = tc.accent_active;
        ui.visuals_mut().widgets.inactive.weak_bg_fill = tc.background;
        ui.visuals_mut().widgets.hovered.weak_bg_fill = tc.accent;
        ui.visuals_mut().widgets.active.weak_bg_fill = tc.accent_active;
        ui.visuals_mut().widgets.open.weak_bg_fill = tc.background;
        ui.visuals_mut().widgets.open.bg_stroke = Stroke::new(1.0, tc.input);

        egui::ComboBox::from_id_salt(id)
            .selected_text(current_label)
            .width(ui.available_width())
            .height(100.0)
            .show_ui(ui, |ui| {
                for (value, label) in options {
                    let is_selected = value == selected;
                    if ui.selectable_label(is_selected, *label).clicked() && !is_selected {
                        *selected = value.clone();
                    }
                }
            })
            .response
    })
    .inner
}
