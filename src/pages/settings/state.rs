use crate::theme::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RendererPreference {
    #[default]
    Wgpu,
    Glow,
}

impl RendererPreference {
    pub fn label(&self) -> &'static str {
        match self {
            RendererPreference::Wgpu => "wgpu (default)",
            RendererPreference::Glow => "glow (OpenGL, better compatibility)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RowDensity {
    Compact,
    #[default]
    Regular,
    Comfy,
}

impl RowDensity {
    pub fn row_height(&self) -> f32 {
        match self {
            RowDensity::Compact => 26.0,
            RowDensity::Regular => 30.0,
            RowDensity::Comfy => 38.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RowDensity::Compact => "Compact",
            RowDensity::Regular => "Regular",
            RowDensity::Comfy => "Comfy",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub theme: Theme,
    pub density: RowDensity,
    pub sidebar_width: f32,
    #[serde(default)]
    pub renderer: RendererPreference,
    #[serde(default = "default_log_keep_days")]
    pub log_keep_days: u64,
}

fn default_log_keep_days() -> u64 {
    7
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            density: RowDensity::Regular,
            sidebar_width: 220.0,
            renderer: RendererPreference::Wgpu,
            log_keep_days: 7,
        }
    }
}

impl SettingsState {
    /// Load from the settings file, falling back to defaults if missing or corrupt.
    pub fn load() -> Self {
        let path = settings_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist current settings to disk. Silently ignores write errors.
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        self.save();
    }
}

fn settings_path() -> String {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        format!("{}\\yssv\\settings.json", base)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/.config/yssv/settings.json", home)
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
        assert_eq!(RowDensity::Comfy.row_height(), 38.0);
    }

    #[test]
    fn settings_serialization_roundtrip() {
        let s = SettingsState {
            theme: Theme::Light,
            density: RowDensity::Compact,
            sidebar_width: 180.0,
            renderer: RendererPreference::Glow,
            log_keep_days: 14,
        };
        let json = serde_json::to_string(&s).unwrap();
        let decoded: SettingsState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.theme, s.theme);
        assert_eq!(decoded.density, s.density);
        assert_eq!(decoded.renderer, RendererPreference::Glow);
        assert_eq!(decoded.log_keep_days, 14);
    }

    #[test]
    fn settings_missing_new_fields_use_defaults() {
        // Simulate a settings file written before renderer/log_keep_days were added.
        let json = r#"{"theme":"Dark","density":"Regular","sidebar_width":220.0}"#;
        let decoded: SettingsState = serde_json::from_str(json).unwrap();
        assert_eq!(decoded.renderer, RendererPreference::Wgpu);
        assert_eq!(decoded.log_keep_days, 7);
    }
}
