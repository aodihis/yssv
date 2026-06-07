use crate::core::results::model::ColumnDef;
use crate::theme::colors;
use egui::{RichText, Ui};

pub fn render_cell(ui: &mut Ui, col: &ColumnDef, value: &Option<String>) {
    match value {
        None => {
            ui.label(
                RichText::new("NULL")
                    .size(12.0)
                    .italics()
                    .color(ui.visuals().weak_text_color()),
            );
        }
        Some(v) => {
            let dtype = col.data_type.to_lowercase();
            if dtype.contains("bool") {
                let is_true = v == "true" || v == "1" || v == "t";
                let (icon, color) = if is_true {
                    ("✓", colors::OK)
                } else {
                    ("✗", ui.visuals().weak_text_color())
                };
                ui.label(
                    RichText::new(format!("{} {}", icon, v))
                        .size(12.0)
                        .color(color),
                );
            } else if is_numeric(&dtype) {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(v)
                            .size(12.0)
                            .font(egui::FontId::monospace(12.0)),
                    );
                });
            } else if dtype.contains("timestamp") || dtype.contains("date") {
                ui.label(
                    RichText::new(v)
                        .size(12.0)
                        .font(egui::FontId::monospace(12.0))
                        .color(ui.visuals().text_color()),
                );
            } else {
                ui.label(RichText::new(v).size(12.0));
            }
        }
    }
}

fn is_numeric(dtype: &str) -> bool {
    dtype.contains("int")
        || dtype.contains("float")
        || dtype.contains("double")
        || dtype.contains("decimal")
        || dtype.contains("numeric")
        || dtype.contains("real")
        || dtype == "int2"
        || dtype == "int4"
        || dtype == "int8"
}
