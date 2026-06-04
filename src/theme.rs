use egui::{Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

// Design token colors
pub mod colors {
    use egui::Color32;

    // Connection label palette (theme-independent)
    pub const RED:    Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
    pub const AMBER:  Color32 = Color32::from_rgb(0xe0, 0x96, 0x2a);
    pub const GREEN:  Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const BLUE:   Color32 = Color32::from_rgb(0x3a, 0x83, 0xf0);
    pub const PURPLE: Color32 = Color32::from_rgb(0x8a, 0x5c, 0xf0);
    pub const GRAY:   Color32 = Color32::from_rgb(0x8b, 0x93, 0xa0);

    // Status
    pub const OK:   Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const WARN: Color32 = Color32::from_rgb(0xd9, 0x83, 0x16);
    pub const ERR:  Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);

    pub mod dark {
        use egui::Color32;
        pub const BG_BASE:     Color32 = Color32::from_rgb(0x11, 0x15, 0x1c);
        pub const BG_PANEL:    Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x17, 0x1c, 0x25);
        pub const BG_HOVER:    Color32 = Color32::from_rgb(0x1a, 0x20, 0x2a);
        pub const BG_ACTIVE:   Color32 = Color32::from_rgb(0x22, 0x2a, 0x36);
        pub const BG_SELECTED: Color32 = Color32::from_rgb(0x16, 0x26, 0x3f);
        pub const TITLEBAR:    Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const GRID_HEADER: Color32 = Color32::from_rgb(0x13, 0x18, 0x20);
        pub const ZEBRA:       Color32 = Color32::from_rgb(0x0e, 0x13, 0x1a);

        pub const BORDER:        Color32 = Color32::from_rgb(0x23, 0x2a, 0x35);
        pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x31, 0x3a, 0x48);
        pub const BORDER_FAINT:  Color32 = Color32::from_rgb(0x1a, 0x20, 0x29);

        pub const TEXT:       Color32 = Color32::from_rgb(0xe7, 0xeb, 0xf2);
        pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x99, 0xa3, 0xb2);
        pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5e, 0x68, 0x77);

        pub const ACCENT:        Color32 = Color32::from_rgb(0x51, 0x81, 0xff);
        pub const ACCENT_HOVER:  Color32 = Color32::from_rgb(0x6f, 0x99, 0xff);
        pub const ACCENT_SOFT:   Color32 = Color32::from_rgb(0x18, 0x26, 0x40);
        pub const ACCENT_BORDER: Color32 = Color32::from_rgb(0x2f, 0x48, 0x85);
    }

    pub mod light {
        use egui::Color32;
        pub const BG_BASE:     Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const BG_PANEL:    Color32 = Color32::from_rgb(0xf5, 0xf6, 0xf8);
        pub const BG_ELEVATED: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const BG_HOVER:    Color32 = Color32::from_rgb(0xee, 0xf0, 0xf3);
        pub const BG_ACTIVE:   Color32 = Color32::from_rgb(0xe6, 0xe9, 0xee);
        pub const BG_SELECTED: Color32 = Color32::from_rgb(0xe9, 0xf0, 0xfe);
        pub const TITLEBAR:    Color32 = Color32::from_rgb(0xf0, 0xf2, 0xf5);
        pub const GRID_HEADER: Color32 = Color32::from_rgb(0xf7, 0xf8, 0xfa);
        pub const ZEBRA:       Color32 = Color32::from_rgb(0xfa, 0xfb, 0xfc);

        pub const BORDER:        Color32 = Color32::from_rgb(0xe3, 0xe6, 0xeb);
        pub const BORDER_STRONG: Color32 = Color32::from_rgb(0xd3, 0xd8, 0xe0);
        pub const BORDER_FAINT:  Color32 = Color32::from_rgb(0xed, 0xef, 0xf2);

        pub const TEXT:       Color32 = Color32::from_rgb(0x1b, 0x1f, 0x24);
        pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x59, 0x62, 0x6f);
        pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x98, 0xa0, 0xac);

        pub const ACCENT:        Color32 = Color32::from_rgb(0x2f, 0x5c, 0xe6);
        pub const ACCENT_HOVER:  Color32 = Color32::from_rgb(0x24, 0x47, 0xc2);
        pub const ACCENT_SOFT:   Color32 = Color32::from_rgb(0xec, 0xf1, 0xfe);
        pub const ACCENT_BORDER: Color32 = Color32::from_rgb(0xbc, 0xcc, 0xfb);
    }
}

pub fn build_visuals(theme: Theme) -> Visuals {
    match theme {
        Theme::Dark => build_dark(),
        Theme::Light => build_light(),
    }
}

fn build_dark() -> Visuals {
    use colors::dark::*;
    let mut v = Visuals::dark();
    v.window_fill = BG_PANEL;
    v.panel_fill = BG_PANEL;
    v.faint_bg_color = BG_HOVER;
    v.extreme_bg_color = GRID_HEADER;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BG_BASE;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = BG_ELEVATED;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = BG_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_BORDER);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = BG_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.open.bg_fill = BG_ACTIVE;
    v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    v
}

fn build_light() -> Visuals {
    use colors::light::*;
    let mut v = Visuals::light();
    v.window_fill = BG_PANEL;
    v.panel_fill = BG_PANEL;
    v.faint_bg_color = BG_HOVER;
    v.extreme_bg_color = GRID_HEADER;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BG_BASE;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = BG_BASE;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = BG_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_BORDER);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = BG_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.corner_radius = egui::CornerRadius::same(5u8);

    v
}

pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    ctx.set_visuals(build_visuals(theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_accent_is_correct() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.selection.stroke.color, colors::dark::ACCENT);
    }

    #[test]
    fn light_accent_is_correct() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.selection.stroke.color, colors::light::ACCENT);
    }

    #[test]
    fn dark_text_is_set() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.override_text_color, Some(colors::dark::TEXT));
    }

    #[test]
    fn light_text_is_set() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.override_text_color, Some(colors::light::TEXT));
    }
}
