use egui::{Margin, Response, TextEdit, Ui};

pub fn text_input(ui: &mut Ui, value: &mut String, placeholder: &str) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(placeholder)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(11, 11)),
    )
}

pub fn password_input(ui: &mut Ui, value: &mut String, placeholder: &str) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(placeholder)
            .password(true)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(11, 11)),
    )
}

pub fn mono_input(ui: &mut Ui, value: &mut String, placeholder: &str) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(placeholder)
            .font(egui::TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(11, 11)),
    )
}
