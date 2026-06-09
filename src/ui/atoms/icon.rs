use egui::{Color32, Response, Ui, Vec2};

pub enum Icon {
    ChevronRight,
    ChevronDown,
    Terminal,
}

impl Icon {
    fn data(&self) -> (&'static [u8], &'static str) {
        match self {
            Icon::ChevronRight => (
                include_bytes!("../../../assets/icons/chevron-right.svg"),
                "bytes://icon/chevron-right.svg",
            ),
            Icon::ChevronDown => (
                include_bytes!("../../../assets/icons/chevron-down.svg"),
                "bytes://icon/chevron-down.svg",
            ),
            Icon::Terminal => (
                include_bytes!("../../../assets/icons/terminal.svg"),
                "bytes://icon/terminal.svg",
            ),
        }
    }
}

pub fn svg_icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (bytes, uri) = icon.data();
    let source = egui::ImageSource::Bytes {
        uri: uri.into(),
        bytes: egui::load::Bytes::Static(bytes),
    };
    ui.add(
        egui::Image::new(source)
            .fit_to_exact_size(Vec2::splat(size))
            .tint(color),
    )
}

pub fn icon_image(icon: Icon, size: f32, color: Color32) -> egui::Image<'static> {
    let (bytes, uri) = icon.data();
    egui::Image::new(egui::ImageSource::Bytes {
        uri: uri.into(),
        bytes: egui::load::Bytes::Static(bytes),
    })
    .fit_to_exact_size(Vec2::splat(size))
    .tint(color)
}
