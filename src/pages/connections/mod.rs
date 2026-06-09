pub mod detail;
pub mod list;
pub mod state;

pub use state::ConnectionsPageState;

pub fn render(ctx: &egui::Context, app: &mut crate::app::YssvApp) {
    let tc = crate::theme::ThemeColors::for_theme(app.settings.theme);

    egui::Panel::left("conn_list_panel")
        .exact_size(307.0)
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame::new().fill(tc.surface))
        .show(ctx, |ui| {
            list::render_list(ui, app, ctx);
            let r = ui.max_rect();
            ui.painter()
                .vline(r.right(), r.y_range(), egui::Stroke::new(1.0, tc.border));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(tc.background))
        .show(ctx, |ui| {
            detail::render_detail(ui, app, ctx);
        });
}
