use serde::{Deserialize, Serialize};
use crate::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowDensity {
    Compact,
    Regular,
    Comfy,
}

impl Default for RowDensity {
    fn default() -> Self {
        RowDensity::Regular
    }
}

impl RowDensity {
    pub fn row_height(&self) -> f32 {
        match self {
            RowDensity::Compact => 26.0,
            RowDensity::Regular => 30.0,
            RowDensity::Comfy   => 38.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RowDensity::Compact => "Compact",
            RowDensity::Regular => "Regular",
            RowDensity::Comfy   => "Comfy",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub theme: Theme,
    pub density: RowDensity,
    pub sidebar_width: f32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            density: RowDensity::Regular,
            sidebar_width: 220.0,
        }
    }
}

impl SettingsState {
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark  => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_theme_dark_to_light() {
        let mut s = SettingsState::default();
        assert_eq!(s.theme, Theme::Dark);
        s.toggle_theme();
        assert_eq!(s.theme, Theme::Light);
        s.toggle_theme();
        assert_eq!(s.theme, Theme::Dark);
    }

    #[test]
    fn density_row_heights() {
        assert_eq!(RowDensity::Compact.row_height(), 26.0);
        assert_eq!(RowDensity::Regular.row_height(), 30.0);
        assert_eq!(RowDensity::Comfy.row_height(),   38.0);
    }

    #[test]
    fn settings_serialization_roundtrip() {
        let s = SettingsState { theme: Theme::Light, density: RowDensity::Compact, sidebar_width: 180.0 };
        let json = serde_json::to_string(&s).unwrap();
        let decoded: SettingsState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.theme, s.theme);
        assert_eq!(decoded.density, s.density);
    }
}
