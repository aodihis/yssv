pub mod main_view;
pub mod sidebar;
pub mod state;

pub use state::{ExplorerState, QueryTab, Tab, TabState, TabView, TableTab};

pub(super) struct LoadRequest {
    pub tab_id: String,
    pub db: String,
    pub schema: String,
    pub table: String,
    pub limit: u32,
    pub offset: u32,
}

pub fn render(ui: &mut egui::Ui, app: &mut crate::app::YssvApp) {
    let sidebar_width = app.settings.sidebar_width;

    let panel_frame = egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    });
    egui::Panel::left("explorer_sidebar")
        .default_size(sidebar_width)
        .size_range(160.0..=400.0)
        .frame(panel_frame)
        .show_inside(ui, |ui| {
            sidebar::render_sidebar(ui, app);
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        main_view::render_main(ui, app);
    });
}
