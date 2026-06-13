use egui::{Response, Ui};

pub fn dropdown<T: PartialEq + Clone, L: AsRef<str>>(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: &mut T,
    options: &[(T, L)],
    selector_height: f32,
) -> Response {
    let current_label = options
        .iter()
        .find(|(v, _)| v == selected)
        .map(|(_, l)| l.as_ref())
        .unwrap_or("—");

    egui::ComboBox::from_id_salt(id)
        .selected_text(current_label)
        .width(ui.available_width())
        .height(selector_height)
        .show_ui(ui, |ui| {
            for (value, label) in options {
                let label = label.as_ref();
                let is_selected = value == selected;
                if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                    *selected = value.clone();
                }
            }
        })
        .response
}
