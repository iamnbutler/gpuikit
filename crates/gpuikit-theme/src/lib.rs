//! A simple theme system for gpui-kit

use gpui::{App, Global, Hsla, SharedString};
use std::collections::HashMap;
use std::sync::Arc;

/// Available theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Dark
    }
}

/// Core theme structure with essential color tokens
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: SharedString,
    pub variant: ThemeVariant,

    /// Foreground color for text
    pub fg: Hsla,

    /// Background color for the main application
    pub bg: Hsla,

    /// Surface color for cards, panels, and elevated surfaces
    pub surface: Hsla,

    /// Border color for dividers and component boundaries
    pub border: Hsla,

    /// Outline color for focus states
    pub outline: Hsla,
}

impl Theme {
    /// Create a Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        Theme {
            name: SharedString::from("Gruvbox Dark"),
            variant: ThemeVariant::Dark,
            fg: parse_hex("#ebdbb2"),
            bg: parse_hex("#282828"),
            surface: parse_hex("#3c3836"),
            border: parse_hex("#504945"),
            outline: parse_hex("#458588"),
        }
    }

    /// Create a Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        Theme {
            name: SharedString::from("Gruvbox Light"),
            variant: ThemeVariant::Light,
            fg: parse_hex("#3c3836"),
            bg: parse_hex("#fbf1c7"),
            surface: parse_hex("#ebdbb2"),
            border: parse_hex("#d5c4a1"),
            outline: parse_hex("#076678"),
        }
    }

    /// Get the global theme instance
    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::gruvbox_dark()
    }
}

/// Global container for the application-wide theme
#[derive(Clone, Debug)]
pub struct GlobalTheme(pub Arc<Theme>);

impl Global for GlobalTheme {}

impl Default for GlobalTheme {
    fn default() -> Self {
        GlobalTheme(Arc::new(Theme::default()))
    }
}

/// Trait for accessing the current theme from an App context
pub trait ActiveTheme {
    /// Returns a reference to the currently active theme
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

/// Simple theme collection manager
#[derive(Debug, Default)]
pub struct Themes {
    themes: HashMap<SharedString, Arc<Theme>>,
    active: Option<SharedString>,
}

impl Themes {
    /// Create a new theme collection with built-in themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Add built-in themes
        let gruvbox_dark = Theme::gruvbox_dark();
        let gruvbox_light = Theme::gruvbox_light();

        themes.insert(gruvbox_dark.name.clone(), Arc::new(gruvbox_dark));
        themes.insert(gruvbox_light.name.clone(), Arc::new(gruvbox_light));

        Self {
            themes,
            active: Some(SharedString::from("Gruvbox Dark")),
        }
    }

    /// Add a custom theme
    pub fn add(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), Arc::new(theme));
    }

    /// Get a theme by name
    pub fn get(&self, name: &str) -> Option<Arc<Theme>> {
        self.themes.get(name).cloned()
    }

    /// Set the active theme by name
    pub fn set_active(&mut self, name: impl Into<SharedString>) -> Option<Arc<Theme>> {
        let name = name.into();
        if let Some(theme) = self.themes.get(&name) {
            self.active = Some(name);
            Some(theme.clone())
        } else {
            None
        }
    }

    /// Get the active theme
    pub fn active(&self) -> Option<Arc<Theme>> {
        self.active
            .as_ref()
            .and_then(|name| self.themes.get(name).cloned())
    }

    /// List all available theme names
    pub fn list(&self) -> Vec<SharedString> {
        self.themes.keys().cloned().collect()
    }

    /// Apply the active theme globally
    pub fn apply_global(&self, cx: &mut App) -> Option<Arc<Theme>> {
        if let Some(theme) = self.active() {
            cx.set_global(GlobalTheme(theme.clone()));
            Some(theme)
        } else {
            None
        }
    }
}

/// Helper function to parse hex color strings
fn parse_hex(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    // Convert RGB to HSLA
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    let saturation = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    gpui::hsla(hue / 360.0, saturation, lightness, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, SharedString::from("Gruvbox Dark"));
        assert_eq!(theme.variant, ThemeVariant::Dark);
    }

    #[test]
    fn test_themes_manager() {
        let mut themes = Themes::new();

        // Should have built-in themes
        assert!(themes.get("Gruvbox Dark").is_some());
        assert!(themes.get("Gruvbox Light").is_some());

        // Active theme should be set
        assert!(themes.active().is_some());

        // Should be able to switch themes
        let light_theme = themes.set_active("Gruvbox Light");
        assert!(light_theme.is_some());
        assert_eq!(themes.active().unwrap().variant, ThemeVariant::Light);

        // Can add custom themes
        let custom = Theme {
            name: SharedString::from("Custom"),
            variant: ThemeVariant::Dark,
            fg: parse_hex("#ffffff"),
            bg: parse_hex("#000000"),
            surface: parse_hex("#111111"),
            border: parse_hex("#222222"),
            outline: parse_hex("#0066cc"),
        };

        themes.add(custom);
        assert!(themes.get("Custom").is_some());
    }

    #[test]
    fn test_hex_parsing() {
        let color = parse_hex("#ffffff");
        assert_eq!(color.l, 1.0); // White should have lightness of 1.0

        let color = parse_hex("#000000");
        assert_eq!(color.l, 0.0); // Black should have lightness of 0.0
    }
}
