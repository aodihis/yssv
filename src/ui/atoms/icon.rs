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
    EyeOff,
    Copy,
    PencilLine,
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
            Icon::EyeOff => (
                include_bytes!("../../../assets/icons/eye-off.svg"),
                "bytes://icon/eye-off.svg",
            ),
            Icon::Copy => (
                include_bytes!("../../../assets/icons/copy.svg"),
                "bytes://icon/copy.svg",
            ),
            Icon::PencilLine => (
                include_bytes!("../../../assets/icons/pencil-line.svg"),
                "bytes://icon/pencil-line.svg",
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

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ICONS: &[Icon] = &[
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Terminal,
        Icon::SquareTerminal,
        Icon::Plug2,
        Icon::CornerDownLeft,
        Icon::Trash2,
        Icon::Database,
        Icon::Layers,
        Icon::Table2,
        Icon::Eye,
        Icon::EyeOff,
        Icon::Copy,
        Icon::PencilLine,
    ];

    #[test]
    fn all_icons_have_non_empty_bytes() {
        for icon in ALL_ICONS {
            let (bytes, _) = icon.data();
            assert!(!bytes.is_empty(), "Icon has empty bytes: {:?}", std::mem::discriminant(icon));
        }
    }

    #[test]
    fn all_icon_uris_start_with_bytes_scheme() {
        for icon in ALL_ICONS {
            let (_, uri) = icon.data();
            assert!(uri.starts_with("bytes://icon/"), "bad URI: {uri}");
        }
    }

    #[test]
    fn all_icon_uris_are_unique() {
        use std::collections::HashSet;
        let uris: HashSet<&str> = ALL_ICONS.iter().map(|i| i.data().1).collect();
        assert_eq!(uris.len(), ALL_ICONS.len(), "duplicate icon URIs detected");
    }

    #[test]
    fn all_icon_bytes_are_valid_svg() {
        for icon in ALL_ICONS {
            let (bytes, uri) = icon.data();
            let text = std::str::from_utf8(bytes)
                .unwrap_or_else(|_| panic!("{uri} is not valid UTF-8"));
            assert!(text.contains("<svg"), "{uri} does not contain <svg");
        }
    }
}
