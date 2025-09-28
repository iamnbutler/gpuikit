//! # A generic theme system for gpui-kit

use gpui::{App, Global, Hsla, SharedString};
use std::sync::Arc;
use utils::color::parse_hex_color;

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

    /// Outline color for focus states (blue from Gruvbox)
    pub outline: Hsla,
}

impl Theme {
    /// Create a Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        Theme {
            name: SharedString::from("Gruvbox Dark"),
            variant: ThemeVariant::Dark,
            fg: parse_hex_color("#ebdbb2").unwrap(),
            bg: parse_hex_color("#282828").unwrap(),
            surface: parse_hex_color("#3c3836").unwrap(),
            border: parse_hex_color("#504945").unwrap(),
            outline: parse_hex_color("#458588").unwrap(),
        }
    }

    /// Create a Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        Theme {
            name: SharedString::from("Gruvbox Light"),
            variant: ThemeVariant::Light,
            fg: parse_hex_color("#3c3836").unwrap(),
            bg: parse_hex_color("#fbf1c7").unwrap(),
            surface: parse_hex_color("#ebdbb2").unwrap(),
            border: parse_hex_color("#d5c4a1").unwrap(),
            outline: parse_hex_color("#076678").unwrap(),
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
