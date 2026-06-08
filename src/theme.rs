use egui::{FontData, FontDefinitions, FontFamily, Stroke, Visuals};

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Plus Jakarta Sans for Proportional
    fonts.font_data.insert(
        "PlusJakarta".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/PlusJakartaSans-VariableFont_wght.ttf"
        ))),
    );

    // IBMPlexSans-Regular for Proportional
    fonts.font_data.insert(
        "IBMPlexSans".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/IBMPlexSans-VariableFont_wdth,wght.ttf"
        ))),
    );

    // IBMPlexMono-Regular for Monospace
    fonts.font_data.insert(
        "IBMPlexMono-Regular".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/IBMPlexMono-Regular.ttf"
        ))),
    );

    {
        let proportional = fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default();

        proportional.insert(0, "PlusJakarta".to_owned());
        proportional.insert(1, "IBMPlexSans".to_owned());
    }

    {
        let monospace = fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default();

        monospace.insert(0, "IBMPlexMono-Regular".to_owned());
    }

    ctx.set_fonts(fonts);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

// Design token colors
pub mod colors {
    use egui::Color32;

    // Connection label palette (theme-independent)
    pub const RED: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
    pub const AMBER: Color32 = Color32::from_rgb(0xe0, 0x96, 0x2a);
    pub const GREEN: Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const BLUE: Color32 = Color32::from_rgb(0x3a, 0x83, 0xf0);
    pub const PURPLE: Color32 = Color32::from_rgb(0x8a, 0x5c, 0xf0);
    pub const GRAY: Color32 = Color32::from_rgb(0x8b, 0x93, 0xa0);

    // Status
    pub const OK: Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const WARN: Color32 = Color32::from_rgb(0xd9, 0x83, 0x16);
    pub const ERR: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);

    pub mod dark {
        use egui::Color32;
        pub const BG_BASE: Color32 = Color32::from_rgb(0x11, 0x15, 0x1c);
        pub const BG_PANEL: Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x17, 0x1c, 0x25);
        pub const BG_HOVER: Color32 = Color32::from_rgb(0x1a, 0x20, 0x2a);
        pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x22, 0x2a, 0x36);
        pub const BG_SELECTED: Color32 = Color32::from_rgb(0x16, 0x26, 0x3f);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const GRID_HEADER: Color32 = Color32::from_rgb(0x13, 0x18, 0x20);
        pub const ZEBRA: Color32 = Color32::from_rgb(0x0e, 0x13, 0x1a);

        pub const BORDER: Color32 = Color32::from_rgb(0x23, 0x2a, 0x35);
        pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x31, 0x3a, 0x48);
        pub const BORDER_FAINT: Color32 = Color32::from_rgb(0x1a, 0x20, 0x29);

        pub const TEXT: Color32 = Color32::from_rgb(0xe7, 0xeb, 0xf2);
        pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x99, 0xa3, 0xb2);
        pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5e, 0x68, 0x77);

        pub const ACCENT: Color32 = Color32::from_rgb(0x51, 0x81, 0xff);
        pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x6f, 0x99, 0xff);
        pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x18, 0x26, 0x40);
        pub const ACCENT_BORDER: Color32 = Color32::from_rgb(0x2f, 0x48, 0x85);
    }

    pub mod light {
        use egui::Color32;
        pub const BG_BASE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const BG_PANEL: Color32 = Color32::from_rgb(0xf5, 0xf6, 0xf8);
        pub const BG_ELEVATED: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const BG_HOVER: Color32 = Color32::from_rgb(0xee, 0xf0, 0xf3);
        pub const BG_ACTIVE: Color32 = Color32::from_rgb(0xe6, 0xe9, 0xee);
        pub const BG_SELECTED: Color32 = Color32::from_rgb(0xe9, 0xf0, 0xfe);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xf5);
        pub const GRID_HEADER: Color32 = Color32::from_rgb(0xf7, 0xf8, 0xfa);
        pub const ZEBRA: Color32 = Color32::from_rgb(0xfa, 0xfb, 0xfc);

        pub const BORDER: Color32 = Color32::from_rgb(0xe3, 0xe6, 0xeb);
        pub const BORDER_STRONG: Color32 = Color32::from_rgb(0xd3, 0xd8, 0xe0);
        pub const BORDER_FAINT: Color32 = Color32::from_rgb(0xed, 0xef, 0xf2);

        pub const TEXT: Color32 = Color32::from_rgb(0x1b, 0x1f, 0x24);
        pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x59, 0x62, 0x6f);
        pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x98, 0xa0, 0xac);

        pub const ACCENT: Color32 = Color32::from_rgb(0x2f, 0x5c, 0xe6);
        pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x24, 0x47, 0xc2);
        pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xec, 0xf1, 0xfe);
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
    ctx.global_style_mut(|style| {
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
    });
}

/// Resolved color set for the current theme — use this in render code instead
/// of checking dark_mode manually everywhere.
pub struct ThemeColors {
    pub bg_base: egui::Color32,
    pub bg_panel: egui::Color32,
    pub bg_elevated: egui::Color32,
    pub bg_hover: egui::Color32,
    pub bg_active: egui::Color32,
    pub bg_selected: egui::Color32,
    pub border: egui::Color32,
    pub border_strong: egui::Color32,
    pub border_faint: egui::Color32,
    pub text: egui::Color32,
    pub text_muted: egui::Color32,
    pub text_faint: egui::Color32,
    pub accent: egui::Color32,
    pub accent_soft: egui::Color32,
    pub grid_header: egui::Color32,
    pub grid_line: egui::Color32,
    pub zebra: egui::Color32,
    pub ok: egui::Color32,
    pub err: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        use colors::dark::*;
        Self {
            bg_base: BG_BASE,
            bg_panel: BG_PANEL,
            bg_elevated: BG_ELEVATED,
            bg_hover: BG_HOVER,
            bg_active: BG_ACTIVE,
            bg_selected: BG_SELECTED,
            border: BORDER,
            border_strong: BORDER_STRONG,
            border_faint: BORDER_FAINT,
            text: TEXT,
            text_muted: TEXT_MUTED,
            text_faint: TEXT_FAINT,
            accent: ACCENT,
            accent_soft: ACCENT_SOFT,
            grid_header: GRID_HEADER,
            grid_line: egui::Color32::from_rgb(0x1c, 0x23, 0x2d),
            zebra: ZEBRA,
            ok: egui::Color32::from_rgb(0x3b, 0xb2, 0x7c),
            err: egui::Color32::from_rgb(0xf0, 0x59, 0x5e),
        }
    }

    pub fn light() -> Self {
        use colors::light::*;
        Self {
            bg_base: BG_BASE,
            bg_panel: BG_PANEL,
            bg_elevated: BG_ELEVATED,
            bg_hover: BG_HOVER,
            bg_active: BG_ACTIVE,
            bg_selected: BG_SELECTED,
            border: BORDER,
            border_strong: BORDER_STRONG,
            border_faint: BORDER_FAINT,
            text: TEXT,
            text_muted: TEXT_MUTED,
            text_faint: TEXT_FAINT,
            accent: ACCENT,
            accent_soft: ACCENT_SOFT,
            grid_header: GRID_HEADER,
            grid_line: egui::Color32::from_rgb(0xed, 0xef, 0xf2),
            zebra: ZEBRA,
            ok: egui::Color32::from_rgb(0x2e, 0xa3, 0x6b),
            err: egui::Color32::from_rgb(0xe5, 0x48, 0x4d),
        }
    }

    /// Derive from the current egui Ui — detects dark vs light by panel fill brightness.
    pub fn from_ui(ui: &egui::Ui) -> Self {
        if ui.visuals().panel_fill.r() < 128 {
            Self::dark()
        } else {
            Self::light()
        }
    }
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
