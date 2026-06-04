fn main() -> eframe::Result<()> {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("YSSV")
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "YSSV",
        options,
        Box::new(|cc| Ok(Box::new(yssv::app::YssvApp::new(cc, rt)))),
    )
}
