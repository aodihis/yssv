use crate::app::YssvApp;
use crate::ui::molecules::alert_dialog::error_dialog;

pub fn render(ui: &mut egui::Ui, app: &mut YssvApp) {
    if let Some(msg) = app.error_modal.clone()
        && error_dialog(ui, "app_error", "Something went wrong", &msg)
    {
        app.error_modal = None;
    }
}
