pub mod detail;
pub mod list;
pub mod state;

pub use state::ConnectionsPageState;

pub fn render(ctx: &egui::Context, app: &mut crate::app::YssvApp) {
    use crate::theme::colors;

    let (panel_bg, bg_base, border_color) = if app.settings.theme == crate::theme::Theme::Dark {
        (
            colors::dark::SURFACE,
            colors::dark::BACKGROUND,
            colors::dark::BORDER,
        )
    } else {
        (
            colors::light::SURFACE,
            colors::light::BACKGROUND,
            colors::light::BORDER,
        )
    };

    egui::Panel::left("conn_list_panel")
        .exact_size(307.0)
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame::new().fill(panel_bg))
        .show(ctx, |ui| {
            list::render_list(ui, app, ctx);
            let r = ui.max_rect();
            ui.painter()
                .vline(r.right(), r.y_range(), egui::Stroke::new(1.0, border_color));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(bg_base))
        .show(ctx, |ui| {
            detail::render_detail(ui, app, ctx);
        });
}
