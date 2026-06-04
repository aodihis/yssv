use egui::{Response, Ui};

pub fn toggle_switch(ui: &mut Ui, enabled: &mut bool) -> Response {
    ui.checkbox(enabled, "")
}
