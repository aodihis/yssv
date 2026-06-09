use egui::{Margin, Response, TextEdit, Ui};

fn base_input<'a>(value: &'a mut String, placeholder: &str, icon: Option<&str>) -> TextEdit<'a> {
    let left: i8 = if icon.is_some() { 28 } else { 11 };
    TextEdit::singleline(value)
        .hint_text(placeholder)
        .desired_width(f32::INFINITY)
        .margin(Margin { left, right: 11, top: 7, bottom: 7 })
}

fn paint_icon(ui: &Ui, resp: &Response, icon: &str) {
    ui.painter().text(
        egui::pos2(resp.rect.left() + 10.0, resp.rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        ui.visuals().weak_text_color(),
    );
}

pub fn text_input(ui: &mut Ui, value: &mut String, placeholder: &str, icon: Option<&str>) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon));
    if let Some(ic) = icon {
        paint_icon(ui, &resp, ic);
    }
    resp
}

pub fn password_input(ui: &mut Ui, value: &mut String, placeholder: &str, icon: Option<&str>) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon).password(true));
    if let Some(ic) = icon { paint_icon(ui, &resp, ic); }
    resp
}

pub fn mono_input(ui: &mut Ui, value: &mut String, placeholder: &str, icon: Option<&str>) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon).font(egui::TextStyle::Monospace));
    if let Some(ic) = icon { paint_icon(ui, &resp, ic); }
    resp
}
