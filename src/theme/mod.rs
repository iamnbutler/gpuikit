//! A trait-based theme system for gpuikit
//!
//! The `Themeable` trait defines the color contract that gpuikit components use.
//! Consumers can implement this trait for their own theme types.

use gpui::{hsla, App, Global, Hsla, SharedString};
use std::sync::Arc;

/// Core theme trait that defines the color contract for UI components.
///
/// Implement this trait for your own theme type to customize colors.
/// Only a small set of "primitive" colors are required - everything else
/// has sensible defaults derived from them.
pub trait Themeable {
    // === Required methods (the primitives) ===

    /// Primary foreground/text color
    fn fg(&self) -> Hsla;

    /// Primary background color
    fn bg(&self) -> Hsla;

    /// Surface color for cards, panels, elevated elements
    fn surface(&self) -> Hsla;

    /// Border color for dividers and boundaries
    fn border(&self) -> Hsla;

    /// Accent color for primary actions and focus states
    fn accent(&self) -> Hsla;

    // === Optional methods with defaults ===

    /// Muted foreground for secondary text
    fn fg_muted(&self) -> Hsla {
        self.fg().opacity(0.7)
    }

    /// Disabled foreground color
    fn fg_disabled(&self) -> Hsla {
        self.fg().opacity(0.4)
    }

    /// Secondary surface for nested panels
    fn surface_secondary(&self) -> Hsla {
        self.surface()
    }

    /// Tertiary surface for deeply nested elements
    fn surface_tertiary(&self) -> Hsla {
        self.surface_secondary()
    }

    /// Secondary border for hover states
    fn border_secondary(&self) -> Hsla {
        self.border()
    }

    /// Subtle border for minimal separation
    fn border_subtle(&self) -> Hsla {
        self.border().opacity(0.5)
    }

    /// Focus outline color
    fn outline(&self) -> Hsla {
        self.accent()
    }

    /// Accent background (for tags, badges)
    fn accent_bg(&self) -> Hsla {
        self.accent().opacity(0.15)
    }

    /// Accent background hover state
    fn accent_bg_hover(&self) -> Hsla {
        self.accent().opacity(0.25)
    }

    /// Danger/error color
    fn danger(&self) -> Hsla {
        hsla(0.0, 0.7, 0.5, 1.0)
    }

    /// Selection highlight color
    fn selection(&self) -> Hsla {
        self.accent().opacity(0.3)
    }

    // === Component-specific defaults ===

    fn button_bg(&self) -> Hsla {
        self.surface()
    }

    fn button_bg_hover(&self) -> Hsla {
        self.surface_secondary()
    }

    fn button_bg_active(&self) -> Hsla {
        self.surface_tertiary()
    }

    fn button_border(&self) -> Hsla {
        self.border()
    }

    fn input_bg(&self) -> Hsla {
        self.surface()
    }

    fn input_border(&self) -> Hsla {
        self.border()
    }

    fn input_border_hover(&self) -> Hsla {
        self.border_secondary()
    }

    fn input_border_focused(&self) -> Hsla {
        self.accent()
    }
}

pub fn init(cx: &mut App) {
    cx.set_global(GlobalTheme::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
}

/// A concrete theme implementation with stored color values.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: SharedString,
    pub variant: ThemeVariant,

    // Primitives
    fg_color: Hsla,
    bg_color: Hsla,
    surface_color: Hsla,
    border_color: Hsla,
    accent_color: Hsla,

    // Overrides (None = use default from trait)
    fg_muted_color: Option<Hsla>,
    fg_disabled_color: Option<Hsla>,
    surface_secondary_color: Option<Hsla>,
    surface_tertiary_color: Option<Hsla>,
    border_secondary_color: Option<Hsla>,
    border_subtle_color: Option<Hsla>,
    outline_color: Option<Hsla>,
    accent_bg_color: Option<Hsla>,
    accent_bg_hover_color: Option<Hsla>,
    danger_color: Option<Hsla>,
    selection_color: Option<Hsla>,
    button_bg_color: Option<Hsla>,
    button_bg_hover_color: Option<Hsla>,
    button_bg_active_color: Option<Hsla>,
    button_border_color: Option<Hsla>,
    input_bg_color: Option<Hsla>,
    input_border_color: Option<Hsla>,
    input_border_hover_color: Option<Hsla>,
    input_border_focused_color: Option<Hsla>,
}

impl Themeable for Theme {
    fn fg(&self) -> Hsla {
        self.fg_color
    }
    fn bg(&self) -> Hsla {
        self.bg_color
    }
    fn surface(&self) -> Hsla {
        self.surface_color
    }
    fn border(&self) -> Hsla {
        self.border_color
    }
    fn accent(&self) -> Hsla {
        self.accent_color
    }

    fn fg_muted(&self) -> Hsla {
        self.fg_muted_color
            .unwrap_or_else(|| self.fg().opacity(0.7))
    }
    fn fg_disabled(&self) -> Hsla {
        self.fg_disabled_color
            .unwrap_or_else(|| self.fg().opacity(0.4))
    }
    fn surface_secondary(&self) -> Hsla {
        self.surface_secondary_color
            .unwrap_or_else(|| self.surface())
    }
    fn surface_tertiary(&self) -> Hsla {
        self.surface_tertiary_color
            .unwrap_or_else(|| self.surface_secondary())
    }
    fn border_secondary(&self) -> Hsla {
        self.border_secondary_color.unwrap_or_else(|| self.border())
    }
    fn border_subtle(&self) -> Hsla {
        self.border_subtle_color
            .unwrap_or_else(|| self.border().opacity(0.5))
    }
    fn outline(&self) -> Hsla {
        self.outline_color.unwrap_or_else(|| self.accent())
    }
    fn accent_bg(&self) -> Hsla {
        self.accent_bg_color
            .unwrap_or_else(|| self.accent().opacity(0.15))
    }
    fn accent_bg_hover(&self) -> Hsla {
        self.accent_bg_hover_color
            .unwrap_or_else(|| self.accent().opacity(0.25))
    }
    fn danger(&self) -> Hsla {
        self.danger_color
            .unwrap_or_else(|| hsla(0.0, 0.7, 0.5, 1.0))
    }
    fn selection(&self) -> Hsla {
        self.selection_color
            .unwrap_or_else(|| self.accent().opacity(0.3))
    }
    fn button_bg(&self) -> Hsla {
        self.button_bg_color.unwrap_or_else(|| self.surface())
    }
    fn button_bg_hover(&self) -> Hsla {
        self.button_bg_hover_color
            .unwrap_or_else(|| self.surface_secondary())
    }
    fn button_bg_active(&self) -> Hsla {
        self.button_bg_active_color
            .unwrap_or_else(|| self.surface_tertiary())
    }
    fn button_border(&self) -> Hsla {
        self.button_border_color.unwrap_or_else(|| self.border())
    }
    fn input_bg(&self) -> Hsla {
        self.input_bg_color.unwrap_or_else(|| self.surface())
    }
    fn input_border(&self) -> Hsla {
        self.input_border_color.unwrap_or_else(|| self.border())
    }
    fn input_border_hover(&self) -> Hsla {
        self.input_border_hover_color
            .unwrap_or_else(|| self.border_secondary())
    }
    fn input_border_focused(&self) -> Hsla {
        self.input_border_focused_color
            .unwrap_or_else(|| self.accent())
    }
}

impl Theme {
    /// Create a new theme with just the required primitives.
    /// All other colors will use sensible defaults.
    pub fn new(
        name: impl Into<SharedString>,
        variant: ThemeVariant,
        fg: Hsla,
        bg: Hsla,
        surface: Hsla,
        border: Hsla,
        accent: Hsla,
    ) -> Self {
        Theme {
            name: name.into(),
            variant,
            fg_color: fg,
            bg_color: bg,
            surface_color: surface,
            border_color: border,
            accent_color: accent,
            fg_muted_color: None,
            fg_disabled_color: None,
            surface_secondary_color: None,
            surface_tertiary_color: None,
            border_secondary_color: None,
            border_subtle_color: None,
            outline_color: None,
            accent_bg_color: None,
            accent_bg_hover_color: None,
            danger_color: None,
            selection_color: None,
            button_bg_color: None,
            button_bg_hover_color: None,
            button_bg_active_color: None,
            button_border_color: None,
            input_bg_color: None,
            input_border_color: None,
            input_border_hover_color: None,
            input_border_focused_color: None,
        }
    }

    /// Create a Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        let mut theme = Theme::new(
            "Gruvbox Dark",
            ThemeVariant::Dark,
            parse_hex("#ebdbb2"), // fg
            parse_hex("#282828"), // bg
            parse_hex("#3c3836"), // surface
            parse_hex("#504945"), // border
            parse_hex("#8ec07c"), // accent
        );
        theme.fg_muted_color = Some(parse_hex("#a89984"));
        theme.fg_disabled_color = Some(parse_hex("#7c6f64"));
        theme.surface_secondary_color = Some(parse_hex("#504945"));
        theme.surface_tertiary_color = Some(parse_hex("#665c54"));
        theme.border_secondary_color = Some(parse_hex("#7c6f64"));
        theme.border_subtle_color = Some(parse_hex("#3c3836"));
        theme.outline_color = Some(parse_hex("#458588"));
        theme.danger_color = Some(parse_hex("#fb4934"));
        theme.selection_color = Some(hsla(55.0 / 360.0, 0.56, 0.64, 0.25));
        theme.button_bg_color = Some(parse_hex("#504945"));
        theme.button_bg_hover_color = Some(parse_hex("#665c54"));
        theme.button_bg_active_color = Some(parse_hex("#7c6f64"));
        theme.button_border_color = Some(parse_hex("#7c6f64"));
        theme.input_border_hover_color = Some(parse_hex("#665c54"));
        theme
    }

    /// Create a Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        let mut theme = Theme::new(
            "Gruvbox Light",
            ThemeVariant::Light,
            parse_hex("#3c3836"), // fg
            parse_hex("#fbf1c7"), // bg
            parse_hex("#ebdbb2"), // surface
            parse_hex("#d5c4a1"), // border
            parse_hex("#427b58"), // accent
        );
        theme.fg_muted_color = Some(parse_hex("#665c54"));
        theme.fg_disabled_color = Some(parse_hex("#a89984"));
        theme.surface_secondary_color = Some(parse_hex("#d5c4a1"));
        theme.surface_tertiary_color = Some(parse_hex("#bdae93"));
        theme.border_secondary_color = Some(parse_hex("#a89984"));
        theme.border_subtle_color = Some(parse_hex("#ebdbb2"));
        theme.outline_color = Some(parse_hex("#076678"));
        theme.danger_color = Some(parse_hex("#cc241d"));
        theme.selection_color = Some(hsla(48.0 / 360.0, 0.87, 0.61, 0.15));
        theme.button_bg_color = Some(parse_hex("#ebdbb2"));
        theme.button_bg_hover_color = Some(parse_hex("#d5c4a1"));
        theme.button_bg_active_color = Some(parse_hex("#bdae93"));
        theme.button_border_color = Some(parse_hex("#a89984"));
        theme.input_border_hover_color = Some(parse_hex("#bdae93"));
        theme
    }

    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::gruvbox_dark()
    }
}

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
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

fn parse_hex(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

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
    fn test_minimal_theme() {
        // Create a theme with just primitives - all else uses defaults
        let theme = Theme::new(
            "Minimal",
            ThemeVariant::Dark,
            parse_hex("#ffffff"),
            parse_hex("#000000"),
            parse_hex("#111111"),
            parse_hex("#333333"),
            parse_hex("#0066cc"),
        );

        // Derived values should work
        assert_eq!(theme.button_bg(), theme.surface());
        assert_eq!(theme.input_border_focused(), theme.accent());
    }

    #[test]
    fn test_hex_parsing() {
        let color = parse_hex("#ffffff");
        assert_eq!(color.l, 1.0);

        let color = parse_hex("#000000");
        assert_eq!(color.l, 0.0);
    }
}
