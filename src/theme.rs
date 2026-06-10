use egui::{FontData, FontDefinitions, FontFamily, Stroke, Visuals};

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "PlusJakarta-Regular".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/PlusJakartaSans-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "PlusJakarta-SemiBold".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/PlusJakartaSans-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "IBMPlexMono-Regular".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/IBMPlexMono-Regular.ttf"
        ))),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "PlusJakarta-Regular".to_owned());

    fonts
        .families
        .entry(FontFamily::Name("SemiBold".into()))
        .or_default()
        .insert(0, "PlusJakarta-SemiBold".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "IBMPlexMono-Regular".to_owned());

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
    pub const ERROR: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);

    // DB engine brand colors (theme-independent)
    pub const POSTGRES: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
    pub const MYSQL: Color32 = Color32::from_rgb(0xc0, 0x82, 0x0f);

    pub mod dark {
        use egui::Color32;

        // Backgrounds
        pub const BACKGROUND: Color32 = Color32::from_rgb(0x11, 0x15, 0x1c);
        pub const SURFACE: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
        pub const SURFACE_SECONDARY: Color32 = Color32::from_rgb(0x1a, 0x20, 0x2a);
        pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(0x22, 0x2a, 0x36);
        pub const SELECTION_BG: Color32 = Color32::from_rgb(0x16, 0x26, 0x3f);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0x0c, 0x10, 0x15);
        pub const TABLE_HEADER: Color32 = Color32::from_rgb(0x13, 0x18, 0x20);
        pub const TABLE_ROW_ALT: Color32 = Color32::from_rgb(0x0e, 0x13, 0x1a);

        // Fields
        pub const FIELD_BG: Color32 = Color32::from_rgb(0x17, 0x1c, 0x25);
        pub const FIELD_BORDER: Color32 = Color32::from_rgb(0x31, 0x3a, 0x48);

        // Borders
        pub const BORDER: Color32 = Color32::from_rgb(0x23, 0x2a, 0x35);
        pub const BORDER_MUTED: Color32 = Color32::from_rgb(0x1a, 0x20, 0x29);
        pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0x2f, 0x48, 0x85);

        // Text
        pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe7, 0xeb, 0xf2);
        pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x99, 0xa3, 0xb2);
        pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x5e, 0x68, 0x77);

        // Controls
        pub const CONTROL_TRACK: Color32 = Color32::from_rgb(0x3a, 0x42, 0x52);

        // Buttons
        pub const BUTTON_PRIMARY_BG: Color32 = Color32::from_rgb(0x51, 0x81, 0xff);
        pub const BUTTON_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x6f, 0x99, 0xff);
        pub const BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(0x18, 0x26, 0x40);
    }

    pub mod light {
        use egui::Color32;

        // Backgrounds
        pub const BACKGROUND: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const SURFACE: Color32 = Color32::from_rgb(0xf5, 0xf6, 0xf8);
        pub const SURFACE_SECONDARY: Color32 = Color32::from_rgb(0xee, 0xf0, 0xf3);
        pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(0xe6, 0xe9, 0xee);
        pub const SELECTION_BG: Color32 = Color32::from_rgb(0xe9, 0xf0, 0xfe);
        pub const TITLEBAR: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xf5);
        pub const TABLE_HEADER: Color32 = Color32::from_rgb(0xf7, 0xf8, 0xfa);
        pub const TABLE_ROW_ALT: Color32 = Color32::from_rgb(0xfa, 0xfb, 0xfc);

        // Fields
        pub const FIELD_BG: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
        pub const FIELD_BORDER: Color32 = Color32::from_rgb(0xd3, 0xd8, 0xe0);

        // Borders
        pub const BORDER: Color32 = Color32::from_rgb(0xe3, 0xe6, 0xeb);
        pub const BORDER_MUTED: Color32 = Color32::from_rgb(0xed, 0xef, 0xf2);
        pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0xbc, 0xcc, 0xfb);

        // Text
        pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x1b, 0x1f, 0x24);
        pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x59, 0x62, 0x6f);
        pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x98, 0xa0, 0xac);

        // Controls
        pub const CONTROL_TRACK: Color32 = Color32::from_rgb(0xc1, 0xc8, 0xd4);

        // Buttons
        pub const BUTTON_PRIMARY_BG: Color32 = Color32::from_rgb(0x2f, 0x5c, 0xe6);
        pub const BUTTON_PRIMARY_HOVER: Color32 = Color32::from_rgb(0x24, 0x47, 0xc2);
        pub const BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(0xec, 0xf1, 0xfe);
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
    v.faint_bg_color = SURFACE_SECONDARY;
    v.extreme_bg_color = TABLE_HEADER;
    v.override_text_color = Some(TEXT_PRIMARY);
    v.selection.bg_fill = SELECTION_BG;
    v.hyperlink_color = BUTTON_PRIMARY_BG;
    v.window_corner_radius = egui::CornerRadius::same(12);

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BACKGROUND;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = BACKGROUND;
    v.widgets.inactive.weak_bg_fill = BACKGROUND;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD_BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = SURFACE_SECONDARY;
    v.widgets.hovered.weak_bg_fill = SURFACE_SECONDARY;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_FOCUS);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = SURFACE_ACTIVE;
    v.widgets.active.weak_bg_fill = SURFACE_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, BUTTON_PRIMARY_BG);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.active.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.open.bg_fill = SURFACE_ACTIVE;
    v.widgets.open.weak_bg_fill = BACKGROUND;
    v.widgets.open.bg_stroke = Stroke::new(1.0, FIELD_BORDER);
    v.text_edit_bg_color = Some(FIELD_BG);
    v
}

fn build_light() -> Visuals {
    use colors::light::*;
    let mut v = Visuals::light();
    v.window_fill = SURFACE;
    v.panel_fill = SURFACE;
    v.faint_bg_color = SURFACE_SECONDARY;
    v.extreme_bg_color = TABLE_HEADER;
    v.override_text_color = Some(TEXT_PRIMARY);
    v.selection.bg_fill = SELECTION_BG;
    v.hyperlink_color = BUTTON_PRIMARY_BG;
    v.window_corner_radius = egui::CornerRadius::same(12);

    let border_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = BACKGROUND;
    v.widgets.noninteractive.bg_stroke = border_stroke;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.inactive.bg_fill = BACKGROUND;
    v.widgets.inactive.weak_bg_fill = BACKGROUND;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD_BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.hovered.bg_fill = SURFACE_SECONDARY;
    v.widgets.hovered.weak_bg_fill = SURFACE_SECONDARY;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_FOCUS);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.active.bg_fill = SURFACE_ACTIVE;
    v.widgets.active.weak_bg_fill = SURFACE_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, BUTTON_PRIMARY_BG);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.active.corner_radius = egui::CornerRadius::same(5u8);

    v.widgets.open.bg_fill = SURFACE_ACTIVE;
    v.widgets.open.weak_bg_fill = BACKGROUND;
    v.widgets.open.bg_stroke = Stroke::new(1.0, FIELD_BORDER);
    v.text_edit_bg_color = Some(FIELD_BG);
    v
}

pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    ctx.set_visuals(build_visuals(theme));

    ctx.global_style_mut(|style| {
        use egui::{FontFamily, FontId, TextStyle};

        style.spacing.button_padding = egui::vec2(14.0, 6.5);
        style.spacing.window_margin = egui::Margin::same(0);
        style.spacing.indent = 15.0;
        style.spacing.interact_size.y = 32.0;
        style.spacing.combo_height = 34.0;

        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
    });
}

/// Resolved color set for the current theme — use this in render code instead
/// of checking dark_mode manually everywhere.
pub struct ThemeColors {
    pub background: egui::Color32,
    pub surface: egui::Color32,
    pub surface_secondary: egui::Color32,
    pub surface_active: egui::Color32,
    pub selection_bg: egui::Color32,
    pub border: egui::Color32,
    pub border_muted: egui::Color32,
    pub field_border: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_disabled: egui::Color32,
    pub control_track: egui::Color32,
    pub button_primary_bg: egui::Color32,
    pub button_secondary_bg: egui::Color32,
    pub success: egui::Color32,
    pub error: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        use colors::dark::*;
        Self {
            background: BACKGROUND,
            surface: SURFACE,
            surface_secondary: SURFACE_SECONDARY,
            surface_active: SURFACE_ACTIVE,
            selection_bg: SELECTION_BG,
            border: BORDER,
            border_muted: BORDER_MUTED,
            field_border: FIELD_BORDER,
            text_primary: TEXT_PRIMARY,
            text_secondary: TEXT_SECONDARY,
            text_disabled: TEXT_DISABLED,
            control_track: CONTROL_TRACK,
            button_primary_bg: BUTTON_PRIMARY_BG,
            button_secondary_bg: BUTTON_SECONDARY_BG,
            success: colors::SUCCESS,
            error: colors::ERROR,
        }
    }

    pub fn light() -> Self {
        use colors::light::*;
        Self {
            background: BACKGROUND,
            surface: SURFACE,
            surface_secondary: SURFACE_SECONDARY,
            surface_active: SURFACE_ACTIVE,
            selection_bg: SELECTION_BG,
            border: BORDER,
            border_muted: BORDER_MUTED,
            field_border: FIELD_BORDER,
            text_primary: TEXT_PRIMARY,
            text_secondary: TEXT_SECONDARY,
            text_disabled: TEXT_DISABLED,
            control_track: CONTROL_TRACK,
            button_primary_bg: BUTTON_PRIMARY_BG,
            button_secondary_bg: BUTTON_SECONDARY_BG,
            success: colors::SUCCESS,
            error: colors::ERROR,
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
    fn dark_selection_fill_is_correct() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.selection.bg_fill, colors::dark::SELECTION_BG);
    }

    #[test]
    fn light_selection_fill_is_correct() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.selection.bg_fill, colors::light::SELECTION_BG);
    }

    #[test]
    fn dark_foreground_is_set() {
        let v = build_visuals(Theme::Dark);
        assert_eq!(v.override_text_color, Some(colors::dark::TEXT_PRIMARY));
    }

    #[test]
    fn light_foreground_is_set() {
        let v = build_visuals(Theme::Light);
        assert_eq!(v.override_text_color, Some(colors::light::TEXT_PRIMARY));
    }
}
