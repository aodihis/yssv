use crate::app::YssvApp;

pub fn render(ctx: &egui::Context, app: &mut YssvApp) {
    if let Some(msg) = app.error_modal.clone() {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(&msg);
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    app.error_modal = None;
                }
            });
    }
}
