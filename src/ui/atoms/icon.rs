use egui::{Color32, Response, Ui, Vec2};

#[derive(Copy, Clone)]
pub enum Icon {
    ChevronRight,
    ChevronDown,
    Terminal,
    SquareTerminal,
    Plug2,
    CornerDownLeft,
    Trash2,
    Database,
    Layers,
    Table2,
    Eye,
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
            Icon::SquareTerminal => (
                include_bytes!("../../../assets/icons/square-terminal.svg"),
                "bytes://icon/square-terminal.svg",
            ),
            Icon::Plug2 => (
                include_bytes!("../../../assets/icons/plug-2.svg"),
                "bytes://icon/plug-2.svg",
            ),
            Icon::CornerDownLeft => (
                include_bytes!("../../../assets/icons/corner-down-left.svg"),
                "bytes://icon/corner-down-left.svg",
            ),
            Icon::Trash2 => (
                include_bytes!("../../../assets/icons/trash-2.svg"),
                "bytes://icon/trash-2.svg",
            ),
            Icon::Database => (
                include_bytes!("../../../assets/icons/database.svg"),
                "bytes://icon/database.svg",
            ),
            Icon::Layers => (
                include_bytes!("../../../assets/icons/layers.svg"),
                "bytes://icon/layers.svg",
            ),
            Icon::Table2 => (
                include_bytes!("../../../assets/icons/table-2.svg"),
                "bytes://icon/table-2.svg",
            ),
            Icon::Eye => (
                include_bytes!("../../../assets/icons/eye.svg"),
                "bytes://icon/eye.svg",
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
