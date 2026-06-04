use egui::{Button, Color32, Response, Ui, Vec2};

pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let accent = if ui.visuals().dark_mode {
        Color32::from_rgb(0x51, 0x81, 0xff)
    } else {
        Color32::from_rgb(0x2f, 0x5c, 0xe6)
    };
    ui.add(
        Button::new(egui::RichText::new(label).color(Color32::WHITE))
            .fill(accent)
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

pub fn ghost_button(ui: &mut Ui, label: &str) -> Response {
    ui.add(
        Button::new(label)
            .fill(Color32::TRANSPARENT)
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

pub fn small_primary_button(ui: &mut Ui, label: &str) -> Response {
    let accent = if ui.visuals().dark_mode {
        Color32::from_rgb(0x51, 0x81, 0xff)
    } else {
        Color32::from_rgb(0x2f, 0x5c, 0xe6)
    };
    ui.add(
        Button::new(egui::RichText::new(label).color(Color32::WHITE).size(12.0))
            .fill(accent)
            .min_size(Vec2::new(0.0, 27.0)),
    )
}
