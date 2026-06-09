pub mod main_view;
pub mod sidebar;
pub mod state;

pub use state::{ExplorerState, TabState, TabView, TableTab};

pub(super) struct LoadRequest {
    pub tab_id: String,
    pub db: String,
    pub schema: String,
    pub table: String,
    pub limit: u32,
    pub offset: u32,
}

pub fn render(ctx: &egui::Context, app: &mut crate::app::YssvApp) {
    let sidebar_width = app.settings.sidebar_width;

    egui::Panel::left("explorer_sidebar")
        .default_size(sidebar_width)
        .size_range(160.0..=400.0)
        .show(ctx, |ui| {
            sidebar::render_sidebar(ui, app, ctx);
        });

    egui::CentralPanel::default()
        .show(ctx, |ui| {
            main_view::render_main(ui, app, ctx);
        });
}
