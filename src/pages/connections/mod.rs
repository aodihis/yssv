pub mod detail;
pub mod list;
pub mod state;

pub use state::ConnectionsPageState;

pub fn render(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let tc = crate::theme::ThemeColors::for_theme(app.settings.theme);

    egui::Panel::left("conn_list_panel")
        .exact_size(307.0)
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame::new().fill(tc.surface))
        .show_inside(ui, |ui| {
            list::render_list(ui, app);
            let r = ui.max_rect();
            ui.painter()
                .vline(r.right(), r.y_range(), egui::Stroke::new(1.0, tc.border));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(tc.background))
        .show_inside(ui, |ui| {
            detail::render_detail(ui, app);
        });

    render_delete_confirm(ui, app);
}

fn render_delete_confirm(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let Some(id) = app.conn_page.pending_delete.clone() else {
        return;
    };
    let conn_name = app
        .conn_page
        .connections
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.name.as_str())
        .unwrap_or("this connection")
        .to_string();

    let tc = crate::theme::ThemeColors::for_theme(app.settings.theme);

    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new("Delete Connection")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(ui.style())
                .fill(tc.surface)
                .stroke(egui::Stroke::new(1.0, tc.border)),
        )
        .show(ui.ctx(), |ui| {
            ui.set_min_width(320.0);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("Delete \"{}\"?", conn_name))
                    .size(13.0)
                    .color(tc.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("This cannot be undone.")
                    .size(12.0)
                    .color(tc.text_secondary),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let del = egui::Button::new(
                        egui::RichText::new("Delete")
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(tc.error)
                    .stroke(egui::Stroke::NONE)
                    .min_size(egui::vec2(72.0, 30.0));
                    if ui.add(del).clicked() {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    let cancel = egui::Button::new(
                        egui::RichText::new("Cancel").size(13.0).color(tc.text_primary),
                    )
                    .fill(tc.background)
                    .stroke(egui::Stroke::new(1.0, tc.field_border))
                    .min_size(egui::vec2(72.0, 30.0));
                    if ui.add(cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
            ui.add_space(4.0);
        });

    if confirmed {
        app.conn_page.pending_delete = None;
        app.delete_connection(&id);
    } else if cancelled {
        app.conn_page.pending_delete = None;
    }
}
