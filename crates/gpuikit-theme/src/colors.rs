//! Color system and utilities for theming
//!
//! This module provides color scales, palettes, and utilities for working
//! with colors in GPUI themes.

use gpui::{hsla, Hsla, Rgba};
use serde::{Deserialize, Serialize};

/// A scale of colors from lightest to darkest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScale {
    /// Lightest shade (50)
    pub shade_50: Hsla,
    /// Very light shade (100)
    pub shade_100: Hsla,
    /// Light shade (200)
    pub shade_200: Hsla,
    /// Light-medium shade (300)
    pub shade_300: Hsla,
    /// Medium-light shade (400)
    pub shade_400: Hsla,
    /// Medium shade (500) - typically the "base" color
    pub shade_500: Hsla,
    /// Medium-dark shade (600)
    pub shade_600: Hsla,
    /// Dark-medium shade (700)
    pub shade_700: Hsla,
    /// Dark shade (800)
    pub shade_800: Hsla,
    /// Very dark shade (900)
    pub shade_900: Hsla,
    /// Darkest shade (950)
    pub shade_950: Hsla,
}

impl ColorScale {
    /// Create a new color scale from a base color
    pub fn from_base(base: Hsla) -> Self {
        let h = base.h;
        let s = base.s;

        Self {
            shade_50: hsla(h, s * 0.3, 0.98, 1.0),
            shade_100: hsla(h, s * 0.4, 0.96, 1.0),
            shade_200: hsla(h, s * 0.5, 0.91, 1.0),
            shade_300: hsla(h, s * 0.6, 0.82, 1.0),
            shade_400: hsla(h, s * 0.7, 0.64, 1.0),
            shade_500: base,
            shade_600: hsla(h, s, 0.41, 1.0),
            shade_700: hsla(h, s * 0.9, 0.32, 1.0),
            shade_800: hsla(h, s * 0.8, 0.26, 1.0),
            shade_900: hsla(h, s * 0.7, 0.21, 1.0),
            shade_950: hsla(h, s * 0.6, 0.13, 1.0),
        }
    }

    /// Get the base color (shade 500)
    pub fn base(&self) -> Hsla {
        self.shade_500
    }

    /// Get a specific shade by index (50, 100, 200, ..., 950)
    pub fn shade(&self, index: u32) -> Option<Hsla> {
        match index {
            50 => Some(self.shade_50),
            100 => Some(self.shade_100),
            200 => Some(self.shade_200),
            300 => Some(self.shade_300),
            400 => Some(self.shade_400),
            500 => Some(self.shade_500),
            600 => Some(self.shade_600),
            700 => Some(self.shade_700),
            800 => Some(self.shade_800),
            900 => Some(self.shade_900),
            950 => Some(self.shade_950),
            _ => None,
        }
    }

    /// Create a grayscale color scale for light themes
    pub fn grayscale_light() -> Self {
        Self {
            shade_50: hsla(0.0, 0.0, 0.98, 1.0),
            shade_100: hsla(0.0, 0.0, 0.96, 1.0),
            shade_200: hsla(0.0, 0.0, 0.91, 1.0),
            shade_300: hsla(0.0, 0.0, 0.82, 1.0),
            shade_400: hsla(0.0, 0.0, 0.64, 1.0),
            shade_500: hsla(0.0, 0.0, 0.45, 1.0),
            shade_600: hsla(0.0, 0.0, 0.32, 1.0),
            shade_700: hsla(0.0, 0.0, 0.25, 1.0),
            shade_800: hsla(0.0, 0.0, 0.15, 1.0),
            shade_900: hsla(0.0, 0.0, 0.09, 1.0),
            shade_950: hsla(0.0, 0.0, 0.04, 1.0),
        }
    }

    /// Create a grayscale color scale for dark themes
    pub fn grayscale_dark() -> Self {
        Self {
            shade_50: hsla(0.0, 0.0, 0.04, 1.0),
            shade_100: hsla(0.0, 0.0, 0.09, 1.0),
            shade_200: hsla(0.0, 0.0, 0.15, 1.0),
            shade_300: hsla(0.0, 0.0, 0.25, 1.0),
            shade_400: hsla(0.0, 0.0, 0.32, 1.0),
            shade_500: hsla(0.0, 0.0, 0.45, 1.0),
            shade_600: hsla(0.0, 0.0, 0.64, 1.0),
            shade_700: hsla(0.0, 0.0, 0.82, 1.0),
            shade_800: hsla(0.0, 0.0, 0.91, 1.0),
            shade_900: hsla(0.0, 0.0, 0.96, 1.0),
            shade_950: hsla(0.0, 0.0, 0.98, 1.0),
        }
    }

    /// Create a blue color scale
    pub fn blue() -> Self {
        Self::from_base(hsla(217.0 / 360.0, 0.91, 0.60, 1.0))
    }

    /// Create a green color scale
    pub fn green() -> Self {
        Self::from_base(hsla(142.0 / 360.0, 0.76, 0.36, 1.0))
    }

    /// Create a red color scale
    pub fn red() -> Self {
        Self::from_base(hsla(0.0, 0.84, 0.60, 1.0))
    }

    /// Create a yellow color scale
    pub fn yellow() -> Self {
        Self::from_base(hsla(45.0 / 360.0, 0.93, 0.47, 1.0))
    }

    /// Create a purple color scale
    pub fn purple() -> Self {
        Self::from_base(hsla(263.0 / 360.0, 0.70, 0.50, 1.0))
    }

    /// Create a cyan color scale
    pub fn cyan() -> Self {
        Self::from_base(hsla(188.0 / 360.0, 0.96, 0.33, 1.0))
    }
}

/// A palette of related colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Primary colors
    pub primary: Vec<Hsla>,
    /// Secondary colors
    pub secondary: Vec<Hsla>,
    /// Tertiary colors
    pub tertiary: Vec<Hsla>,
    /// Neutral colors
    pub neutral: Vec<Hsla>,
}

impl ColorPalette {
    /// Create a new color palette
    pub fn new(
        primary: Vec<Hsla>,
        secondary: Vec<Hsla>,
        tertiary: Vec<Hsla>,
        neutral: Vec<Hsla>,
    ) -> Self {
        Self {
            primary,
            secondary,
            tertiary,
            neutral,
        }
    }

    /// Create a monochromatic palette from a base color
    pub fn monochromatic(base: Hsla) -> Self {
        let scale = ColorScale::from_base(base);
        let colors = vec![
            scale.shade_300,
            scale.shade_400,
            scale.shade_500,
            scale.shade_600,
            scale.shade_700,
        ];

        Self {
            primary: colors.clone(),
            secondary: vec![],
            tertiary: vec![],
            neutral: ColorScale::grayscale_light().to_vec(),
        }
    }

    /// Create a complementary palette
    pub fn complementary(base: Hsla) -> Self {
        let complement = hsla((base.h + 0.5) % 1.0, base.s, base.l, base.a);

        Self {
            primary: ColorScale::from_base(base).to_vec(),
            secondary: ColorScale::from_base(complement).to_vec(),
            tertiary: vec![],
            neutral: ColorScale::grayscale_light().to_vec(),
        }
    }

    /// Create a triadic palette
    pub fn triadic(base: Hsla) -> Self {
        let second = hsla((base.h + 0.333) % 1.0, base.s, base.l, base.a);
        let third = hsla((base.h + 0.667) % 1.0, base.s, base.l, base.a);

        Self {
            primary: ColorScale::from_base(base).to_vec(),
            secondary: ColorScale::from_base(second).to_vec(),
            tertiary: ColorScale::from_base(third).to_vec(),
            neutral: ColorScale::grayscale_light().to_vec(),
        }
    }
}

impl ColorScale {
    /// Convert to a vector of colors
    pub fn to_vec(&self) -> Vec<Hsla> {
        vec![
            self.shade_50,
            self.shade_100,
            self.shade_200,
            self.shade_300,
            self.shade_400,
            self.shade_500,
            self.shade_600,
            self.shade_700,
            self.shade_800,
            self.shade_900,
            self.shade_950,
        ]
    }
}

/// System colors for platform integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemColors {
    /// Window background
    pub window: Hsla,
    /// Window foreground/text
    pub window_text: Hsla,
    /// Control background (buttons, inputs)
    pub control: Hsla,
    /// Control foreground/text
    pub control_text: Hsla,
    /// Selection/highlight background
    pub selection: Hsla,
    /// Selection/highlight text
    pub selection_text: Hsla,
    /// Focus indicator
    pub focus: Hsla,
    /// Link color
    pub link: Hsla,
    /// Visited link color
    pub link_visited: Hsla,
}

impl SystemColors {
    /// Default system colors for light theme
    pub fn default_light() -> Self {
        Self {
            window: hsla(0.0, 0.0, 1.0, 1.0),
            window_text: hsla(0.0, 0.0, 0.13, 1.0),
            control: hsla(0.0, 0.0, 0.96, 1.0),
            control_text: hsla(0.0, 0.0, 0.13, 1.0),
            selection: hsla(217.0 / 360.0, 0.91, 0.60, 0.3),
            selection_text: hsla(0.0, 0.0, 0.13, 1.0),
            focus: hsla(217.0 / 360.0, 0.91, 0.60, 1.0),
            link: hsla(217.0 / 360.0, 0.91, 0.50, 1.0),
            link_visited: hsla(263.0 / 360.0, 0.70, 0.50, 1.0),
        }
    }

    /// Default system colors for dark theme
    pub fn default_dark() -> Self {
        Self {
            window: hsla(0.0, 0.0, 0.09, 1.0),
            window_text: hsla(0.0, 0.0, 0.91, 1.0),
            control: hsla(0.0, 0.0, 0.13, 1.0),
            control_text: hsla(0.0, 0.0, 0.91, 1.0),
            selection: hsla(217.0 / 360.0, 0.91, 0.60, 0.3),
            selection_text: hsla(0.0, 0.0, 0.91, 1.0),
            focus: hsla(217.0 / 360.0, 0.91, 0.60, 1.0),
            link: hsla(217.0 / 360.0, 0.91, 0.70, 1.0),
            link_visited: hsla(263.0 / 360.0, 0.70, 0.70, 1.0),
        }
    }
}

/// Color manipulation utilities
pub mod utils {
    use super::*;

    /// Darken a color by a percentage (0.0 to 1.0)
    pub fn darken(color: Hsla, amount: f32) -> Hsla {
        let l = (color.l - amount).max(0.0);
        hsla(color.h, color.s, l, color.a)
    }

    /// Lighten a color by a percentage (0.0 to 1.0)
    pub fn lighten(color: Hsla, amount: f32) -> Hsla {
        let l = (color.l + amount).min(1.0);
        hsla(color.h, color.s, l, color.a)
    }

    /// Saturate a color by a percentage (0.0 to 1.0)
    pub fn saturate(color: Hsla, amount: f32) -> Hsla {
        let s = (color.s + amount).min(1.0);
        hsla(color.h, s, color.l, color.a)
    }

    /// Desaturate a color by a percentage (0.0 to 1.0)
    pub fn desaturate(color: Hsla, amount: f32) -> Hsla {
        let s = (color.s - amount).max(0.0);
        hsla(color.h, s, color.l, color.a)
    }

    /// Adjust alpha/opacity
    pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
        hsla(color.h, color.s, color.l, alpha)
    }

    /// Mix two colors
    pub fn mix(color1: Hsla, color2: Hsla, weight: f32) -> Hsla {
        let w = weight.clamp(0.0, 1.0);
        let w2 = 1.0 - w;

        hsla(
            color1.h * w + color2.h * w2,
            color1.s * w + color2.s * w2,
            color1.l * w + color2.l * w2,
            color1.a * w + color2.a * w2,
        )
    }

    /// Create a color from hex string
    pub fn from_hex(hex: &str) -> Result<Hsla, String> {
        use csscolorparser::Color;

        let color = Color::from_html(hex).map_err(|e| format!("Invalid hex color: {}", e))?;

        let [r, g, b, a] = color.to_rgba8();
        let rgba = Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        };

        Ok(rgba.into())
    }

    /// Convert color to hex string
    pub fn to_hex(color: Hsla) -> String {
        let rgba: Rgba = color.into();
        format!(
            "#{:02x}{:02x}{:02x}",
            (rgba.r * 255.0) as u8,
            (rgba.g * 255.0) as u8,
            (rgba.b * 255.0) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scale() {
        let scale = ColorScale::blue();
        assert!(scale.shade(500).is_some());
        assert!(scale.shade(999).is_none());
    }

    #[test]
    fn test_color_manipulation() {
        let color = hsla(0.5, 0.5, 0.5, 1.0);
        let darker = utils::darken(color, 0.1);
        assert!(darker.l < color.l);

        let lighter = utils::lighten(color, 0.1);
        assert!(lighter.l > color.l);
    }
}
