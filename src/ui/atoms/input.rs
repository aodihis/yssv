use eframe::epaint::Color32;
use egui::{Margin, Response, RichText, TextEdit, Ui, Vec2};

fn base_input<'a>(value: &'a mut String, placeholder: &str, icon: Option<&str>) -> TextEdit<'a> {
    let left: i8 = if icon.is_some() { 28 } else { 11 };
    TextEdit::singleline(value)
        .hint_text(placeholder)
        .desired_width(f32::INFINITY)
        .margin(Margin {
            left,
            right: 11,
            top: 7,
            bottom: 7,
        })
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

pub fn text_input(
    ui: &mut Ui,
    value: &mut String,
    placeholder: &str,
    icon: Option<&str>,
) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon));
    if let Some(ic) = icon {
        paint_icon(ui, &resp, ic);
    }
    resp
}

pub fn password_input(
    ui: &mut Ui,
    value: &mut String,
    placeholder: &str,
    icon: Option<&str>,
) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon).password(true));
    if let Some(ic) = icon {
        paint_icon(ui, &resp, ic);
    }
    resp
}

pub fn mono_input(
    ui: &mut Ui,
    value: &mut String,
    placeholder: &str,
    icon: Option<&str>,
) -> Response {
    let resp = ui.add(base_input(value, placeholder, icon).font(egui::TextStyle::Monospace));
    if let Some(ic) = icon {
        paint_icon(ui, &resp, ic);
    }
    resp
}

/// Text input + "Browse" button for selecting a file path.
/// The text field fills available width; the button opens a native file dialog.
pub fn file_input(ui: &mut Ui, value: &mut String, placeholder: &str) -> Response {
    let tc = crate::theme::ThemeColors::from_ui(ui);
    const BTN_W: f32 = 80.0;
    const GAP: f32 = 8.0;

    let mut text_resp: Option<Response> = None;

    ui.horizontal(|ui| {
        let text_w = (ui.available_width() - BTN_W - GAP).max(0.0);
        ui.set_width(ui.available_width());

        text_resp = Some(
            ui.add(
                TextEdit::singleline(value)
                    .hint_text(placeholder)
                    .desired_width(text_w)
                    .margin(Margin {
                        left: 11,
                        right: 11,
                        top: 7,
                        bottom: 7,
                    }),
            ),
        );

        ui.add_space(GAP);

        ui.scope(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(0.0, 6.0);
            ui.spacing_mut().interact_size = Vec2::new(BTN_W, 30.0);
            let btn = egui::Button::new(RichText::new("Browse").size(12.5).color(tc.text_primary))
                .fill(Color32::WHITE)
                .min_size(Vec2::new(BTN_W, 30.0));
            if ui.add(btn).clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file() {
                    *value = path.to_string_lossy().into_owned();
                }
        });
    });

    text_resp.unwrap()
}
