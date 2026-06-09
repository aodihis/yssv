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

pub mod colors {
    use egui::Color32;

    // Connection label palette (theme-independent)
    pub const RED: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
    pub const AMBER: Color32 = Color32::from_rgb(0xe0, 0x96, 0x2a);
    pub const GREEN: Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const BLUE: Color32 = Color32::from_rgb(0x3a, 0x83, 0xf0);
    pub const PURPLE: Color32 = Color32::from_rgb(0x8a, 0x5c, 0xf0);
    pub const GRAY: Color32 = Color32::from_rgb(0x8b, 0x93, 0xa0);

    // Semantic status (theme-independent)
    pub const SUCCESS: Color32 = Color32::from_rgb(0x2e, 0xa3, 0x6b);
    pub const WARNING: Color32 = Color32::from_rgb(0xd9, 0x83, 0x16);
    pub const DESTRUCTIVE: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);

    pub mod dark {
        use egui::Color32;

        // Backgrounds
        pub const BACKGROUND: Color32 = Color32::from_rgb(0x11, 0x15, 0x1c);
        pub const SURFACE: Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const CARD: Color32 = Color32::from_rgb(0x17, 0x1c, 0x25);
        pub const ACCENT: Color32 = Color32::from_rgb(0x1a, 0x20, 0x2a);
        pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0x22, 0x2a, 0x36);
        pub const PRIMARY_SUBTLE: Color32 = Color32::from_rgb(0x16, 0x26, 0x3f);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const TABLE_HEADER: Color32 = Color32::from_rgb(0x13, 0x18, 0x20);
        pub const TABLE_ROW_ALT: Color32 = Color32::from_rgb(0x0e, 0x13, 0x1a);

        // Borders
        pub const BORDER: Color32 = Color32::from_rgb(0x23, 0x2a, 0x35);
        pub const INPUT: Color32 = Color32::from_rgb(0x31, 0x3a, 0x48);
        pub const BORDER_MUTED: Color32 = Color32::from_rgb(0x1a, 0x20, 0x29);

        // Text
        pub const FOREGROUND: Color32 = Color32::from_rgb(0xe7, 0xeb, 0xf2);
        pub const MUTED_FOREGROUND: Color32 = Color32::from_rgb(0x99, 0xa3, 0xb2);
        pub const SUBTLE_FOREGROUND: Color32 = Color32::from_rgb(0x5e, 0x68, 0x77);

        // Primary (action/brand color)
        pub const PRIMARY: Color32 = Color32::from_rgb(0x51, 0x81, 0xff);
        pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x6f, 0x99, 0xff);
        pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(0x18, 0x26, 0x40);
        pub const PRIMARY_BORDER: Color32 = Color32::from_rgb(0x2f, 0x48, 0x85);
    }

    pub mod light {
        use egui::Color32;

        // Backgrounds
        pub const BACKGROUND: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const SURFACE: Color32 = Color32::from_rgb(0xf5, 0xf6, 0xf8);
        pub const CARD: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const ACCENT: Color32 = Color32::from_rgb(0xee, 0xf0, 0xf3);
        pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0xe6, 0xe9, 0xee);
        pub const PRIMARY_SUBTLE: Color32 = Color32::from_rgb(0xe9, 0xf0, 0xfe);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xf5);
        pub const TABLE_HEADER: Color32 = Color32::from_rgb(0xf7, 0xf8, 0xfa);
        pub const TABLE_ROW_ALT: Color32 = Color32::from_rgb(0xfa, 0xfb, 0xfc);

        // Borders
        pub const BORDER: Color32 = Color32::from_rgb(0xe3, 0xe6, 0xeb);
        pub const INPUT: Color32 = Color32::from_rgb(0xd3, 0xd8, 0xe0);
        pub const BORDER_MUTED: Color32 = Color32::from_rgb(0xed, 0xef, 0xf2);

        // Text
        pub const FOREGROUND: Color32 = Color32::from_rgb(0x1b, 0x1f, 0x24);
        pub const MUTED_FOREGROUND: Color32 = Color32::from_rgb(0x59, 0x62, 0x6f);
        pub const SUBTLE_FOREGROUND: Color32 = Color32::from_rgb(0x98, 0xa0, 0xac);

        // Primary (action/brand color)
        pub const PRIMARY: Color32 = Color32::from_rgb(0x2f, 0x5c, 0xe6);
        pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x24, 0x47, 0xc2);
        pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(0xec, 0xf1, 0xfe);
        pub const PRIMARY_BORDER: Color32 = Color32::from_rgb(0xbc, 0xcc, 0xfb);
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
    v.window_fill = SURFACE;
    v.panel_fill = SURFACE;
    v.faint_bg_color = ACCENT;
    v.extreme_bg_color = TABLE_HEADER;
    v.override_text_color = Some(FOREGROUND);
    v.selection.bg_fill = PRIMARY_MUTED;
    v.selection.stroke = Stroke::new(1.0, PRIMARY);
    v.hyperlink_color = PRIMARY;

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BACKGROUND;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, FOREGROUND);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = CARD;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, INPUT);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, MUTED_FOREGROUND);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = ACCENT;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, PRIMARY_BORDER);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, FOREGROUND);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = ACCENT_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY);
    v.widgets.active.fg_stroke = Stroke::new(1.0, FOREGROUND);
    v.widgets.active.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.open.bg_fill = ACCENT_ACTIVE;
    v.widgets.open.bg_stroke = Stroke::new(1.0, PRIMARY);
    v
}

fn build_light() -> Visuals {
    use colors::light::*;
    let mut v = Visuals::light();
    v.window_fill = SURFACE;
    v.panel_fill = SURFACE;
    v.faint_bg_color = ACCENT;
    v.extreme_bg_color = TABLE_HEADER;
    v.override_text_color = Some(FOREGROUND);
    v.selection.bg_fill = PRIMARY_MUTED;
    v.selection.stroke = Stroke::new(1.0, PRIMARY);
    v.hyperlink_color = PRIMARY;

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BACKGROUND;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, FOREGROUND);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = BACKGROUND;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, INPUT);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, MUTED_FOREGROUND);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = ACCENT;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, PRIMARY_BORDER);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, FOREGROUND);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = ACCENT_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY);
    v.widgets.active.fg_stroke = Stroke::new(1.0, FOREGROUND);
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
    pub background: egui::Color32,
    pub surface: egui::Color32,
    pub card: egui::Color32,
    pub accent: egui::Color32,
    pub accent_active: egui::Color32,
    pub primary_subtle: egui::Color32,
    pub border: egui::Color32,
    pub input: egui::Color32,
    pub border_muted: egui::Color32,
    pub foreground: egui::Color32,
    pub muted_foreground: egui::Color32,
    pub subtle_foreground: egui::Color32,
    pub primary: egui::Color32,
    pub primary_muted: egui::Color32,
    pub table_header: egui::Color32,
    pub table_border: egui::Color32,
    pub table_row_alt: egui::Color32,
    pub success: egui::Color32,
    pub destructive: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        use colors::dark::*;
        Self {
            background: BACKGROUND,
            surface: SURFACE,
            card: CARD,
            accent: ACCENT,
            accent_active: ACCENT_ACTIVE,
            primary_subtle: PRIMARY_SUBTLE,
            border: BORDER,
            input: INPUT,
            border_muted: BORDER_MUTED,
            foreground: FOREGROUND,
            muted_foreground: MUTED_FOREGROUND,
            subtle_foreground: SUBTLE_FOREGROUND,
            primary: PRIMARY,
            primary_muted: PRIMARY_MUTED,
            table_header: TABLE_HEADER,
            table_border: BORDER_MUTED,
            table_row_alt: TABLE_ROW_ALT,
            success: colors::SUCCESS,
            destructive: colors::DESTRUCTIVE,
        }
    }

    pub fn light() -> Self {
        use colors::light::*;
        Self {
            background: BACKGROUND,
            surface: SURFACE,
            card: CARD,
            accent: ACCENT,
            accent_active: ACCENT_ACTIVE,
            primary_subtle: PRIMARY_SUBTLE,
            border: BORDER,
            input: INPUT,
            border_muted: BORDER_MUTED,
            foreground: FOREGROUND,
            muted_foreground: MUTED_FOREGROUND,
            subtle_foreground: SUBTLE_FOREGROUND,
            primary: PRIMARY,
            primary_muted: PRIMARY_MUTED,
            table_header: TABLE_HEADER,
            table_border: BORDER_MUTED,
            table_row_alt: TABLE_ROW_ALT,
            success: colors::SUCCESS,
            destructive: colors::DESTRUCTIVE,
        }
    }

    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark(),
            Theme::Light => Self::light(),
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
    fn dark_primary_is_correct() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.selection.stroke.color, colors::dark::PRIMARY);
    }

    #[test]
    fn light_primary_is_correct() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.selection.stroke.color, colors::light::PRIMARY);
    }

    #[test]
    fn dark_foreground_is_set() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.override_text_color, Some(colors::dark::FOREGROUND));
    }

    #[test]
    fn light_foreground_is_set() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.override_text_color, Some(colors::light::FOREGROUND));
    }
}
