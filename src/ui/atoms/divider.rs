use egui::Ui;

pub fn h_divider(ui: &mut Ui) {
    ui.add(egui::Separator::default().horizontal().spacing(0.0));
}

pub fn v_divider(ui: &mut Ui) {
    ui.add(egui::Separator::default().vertical().spacing(0.0));
}
