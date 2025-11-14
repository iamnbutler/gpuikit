//! Style definitions and utilities for theming
//!
//! This module provides typography, spacing, borders, and other style-related
//! types for building consistent themes.

use gpui::{px, Hsla, Pixels};
use serde::{Deserialize, Serialize};

/// Typography settings for text rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    /// Font family stack
    pub font_family: FontFamily,
    /// Base font size
    pub font_size_base: Pixels,
    /// Font size scale
    pub font_scale: FontScale,
    /// Line height settings
    pub line_height: LineHeight,
    /// Font weight definitions
    pub font_weights: FontWeights,
    /// Letter spacing settings
    pub letter_spacing: LetterSpacing,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: FontFamily::default(),
            font_size_base: px(14.0),
            font_scale: FontScale::default(),
            line_height: LineHeight::default(),
            font_weights: FontWeights::default(),
            letter_spacing: LetterSpacing::default(),
        }
    }
}

/// Font family definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    /// Sans-serif font stack
    pub sans: Vec<String>,
    /// Serif font stack
    pub serif: Vec<String>,
    /// Monospace font stack
    pub mono: Vec<String>,
    /// Display font stack
    pub display: Vec<String>,
}

impl Default for FontFamily {
    fn default() -> Self {
        Self {
            sans: vec![
                "Inter".to_string(),
                "-apple-system".to_string(),
                "BlinkMacSystemFont".to_string(),
                "Segoe UI".to_string(),
                "Roboto".to_string(),
                "Helvetica Neue".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string(),
            ],
            serif: vec![
                "Georgia".to_string(),
                "Times New Roman".to_string(),
                "serif".to_string(),
            ],
            mono: vec![
                "JetBrains Mono".to_string(),
                "Fira Code".to_string(),
                "Consolas".to_string(),
                "Monaco".to_string(),
                "Courier New".to_string(),
                "monospace".to_string(),
            ],
            display: vec![
                "Inter Display".to_string(),
                "SF Pro Display".to_string(),
                "Helvetica Neue".to_string(),
                "sans-serif".to_string(),
            ],
        }
    }
}

/// Font size scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontScale {
    /// Extra small text
    pub xs: f32,
    /// Small text
    pub sm: f32,
    /// Base/normal text
    pub base: f32,
    /// Large text
    pub lg: f32,
    /// Extra large text
    pub xl: f32,
    /// 2x large text
    pub xl2: f32,
    /// 3x large text
    pub xl3: f32,
    /// 4x large text
    pub xl4: f32,
    /// 5x large text
    pub xl5: f32,
}

impl Default for FontScale {
    fn default() -> Self {
        Self {
            xs: 0.75,   // 12px at 16px base
            sm: 0.875,  // 14px at 16px base
            base: 1.0,  // 16px
            lg: 1.125,  // 18px at 16px base
            xl: 1.25,   // 20px at 16px base
            xl2: 1.5,   // 24px at 16px base
            xl3: 1.875, // 30px at 16px base
            xl4: 2.25,  // 36px at 16px base
            xl5: 3.0,   // 48px at 16px base
        }
    }
}

/// Line height settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHeight {
    /// Tight line height (1.25)
    pub tight: f32,
    /// Normal line height (1.5)
    pub normal: f32,
    /// Relaxed line height (1.75)
    pub relaxed: f32,
    /// Loose line height (2.0)
    pub loose: f32,
}

impl Default for LineHeight {
    fn default() -> Self {
        Self {
            tight: 1.25,
            normal: 1.5,
            relaxed: 1.75,
            loose: 2.0,
        }
    }
}

/// Font weight definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontWeights {
    /// Thin weight (100)
    pub thin: u16,
    /// Extra light weight (200)
    pub extra_light: u16,
    /// Light weight (300)
    pub light: u16,
    /// Normal weight (400)
    pub normal: u16,
    /// Medium weight (500)
    pub medium: u16,
    /// Semi-bold weight (600)
    pub semi_bold: u16,
    /// Bold weight (700)
    pub bold: u16,
    /// Extra bold weight (800)
    pub extra_bold: u16,
    /// Black weight (900)
    pub black: u16,
}

impl Default for FontWeights {
    fn default() -> Self {
        Self {
            thin: 100,
            extra_light: 200,
            light: 300,
            normal: 400,
            medium: 500,
            semi_bold: 600,
            bold: 700,
            extra_bold: 800,
            black: 900,
        }
    }
}

/// Letter spacing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetterSpacing {
    /// Tighter letter spacing
    pub tighter: Pixels,
    /// Tight letter spacing
    pub tight: Pixels,
    /// Normal letter spacing
    pub normal: Pixels,
    /// Wide letter spacing
    pub wide: Pixels,
    /// Wider letter spacing
    pub wider: Pixels,
}

impl Default for LetterSpacing {
    fn default() -> Self {
        Self {
            tighter: px(-0.05),
            tight: px(-0.025),
            normal: px(0.0),
            wide: px(0.025),
            wider: px(0.05),
        }
    }
}

/// Spacing system for consistent layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacing {
    /// Extra small spacing
    pub xs: Pixels,
    /// Small spacing
    pub sm: Pixels,
    /// Medium spacing
    pub md: Pixels,
    /// Large spacing
    pub lg: Pixels,
    /// Extra large spacing
    pub xl: Pixels,
    /// 2x large spacing
    pub xl2: Pixels,
    /// 3x large spacing
    pub xl3: Pixels,
    /// 4x large spacing
    pub xl4: Pixels,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: px(2.0),
            sm: px(4.0),
            md: px(8.0),
            lg: px(16.0),
            xl: px(24.0),
            xl2: px(32.0),
            xl3: px(48.0),
            xl4: px(64.0),
        }
    }
}

impl Spacing {
    /// Get spacing value by size name
    pub fn get(&self, size: &str) -> Option<Pixels> {
        match size {
            "xs" => Some(self.xs),
            "sm" => Some(self.sm),
            "md" => Some(self.md),
            "lg" => Some(self.lg),
            "xl" => Some(self.xl),
            "2xl" => Some(self.xl2),
            "3xl" => Some(self.xl3),
            "4xl" => Some(self.xl4),
            _ => None,
        }
    }

    /// Create uniform padding/margin
    pub fn uniform(&self, size: &str) -> Option<BoxSpacing> {
        self.get(size).map(|value| BoxSpacing {
            top: value,
            right: value,
            bottom: value,
            left: value,
        })
    }

    /// Create vertical padding/margin
    pub fn vertical(&self, size: &str) -> Option<BoxSpacing> {
        self.get(size).map(|value| BoxSpacing {
            top: value,
            right: px(0.0),
            bottom: value,
            left: px(0.0),
        })
    }

    /// Create horizontal padding/margin
    pub fn horizontal(&self, size: &str) -> Option<BoxSpacing> {
        self.get(size).map(|value| BoxSpacing {
            top: px(0.0),
            right: value,
            bottom: px(0.0),
            left: value,
        })
    }
}

/// Box spacing for padding/margin
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoxSpacing {
    pub top: Pixels,
    pub right: Pixels,
    pub bottom: Pixels,
    pub left: Pixels,
}

impl Default for BoxSpacing {
    fn default() -> Self {
        Self {
            top: px(0.0),
            right: px(0.0),
            bottom: px(0.0),
            left: px(0.0),
        }
    }
}

/// Border style definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderStyle {
    /// Border width
    pub width: Pixels,
    /// Border color
    pub color: Hsla,
    /// Border radius
    pub radius: BorderRadius,
    /// Border style type
    #[serde(default)]
    pub style: BorderType,
}

/// Border radius definition
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BorderRadius {
    /// Top-left radius
    pub top_left: Pixels,
    /// Top-right radius
    pub top_right: Pixels,
    /// Bottom-right radius
    pub bottom_right: Pixels,
    /// Bottom-left radius
    pub bottom_left: Pixels,
}

impl BorderRadius {
    /// Create uniform border radius
    pub fn uniform(radius: Pixels) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// No border radius
    pub fn none() -> Self {
        Self::uniform(px(0.0))
    }

    /// Small border radius
    pub fn small() -> Self {
        Self::uniform(px(2.0))
    }

    /// Medium border radius
    pub fn medium() -> Self {
        Self::uniform(px(4.0))
    }

    /// Large border radius
    pub fn large() -> Self {
        Self::uniform(px(8.0))
    }

    /// Extra large border radius
    pub fn xl() -> Self {
        Self::uniform(px(12.0))
    }

    /// Fully rounded (pill shape)
    pub fn full() -> Self {
        Self::uniform(px(9999.0))
    }
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self::medium()
    }
}

/// Border type/style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderType {
    Solid,
    Dashed,
    Dotted,
    Double,
    None,
}

impl Default for BorderType {
    fn default() -> Self {
        Self::Solid
    }
}

/// Shadow definition for elevation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    /// Horizontal offset
    pub x: Pixels,
    /// Vertical offset
    pub y: Pixels,
    /// Blur radius
    pub blur: Pixels,
    /// Spread radius
    pub spread: Pixels,
    /// Shadow color
    pub color: Hsla,
}

impl Shadow {
    /// Create a new shadow
    pub fn new(x: Pixels, y: Pixels, blur: Pixels, spread: Pixels, color: Hsla) -> Self {
        Self {
            x,
            y,
            blur,
            spread,
            color,
        }
    }

    /// Small elevation shadow
    pub fn small(color: Hsla) -> Self {
        Self::new(px(0.0), px(1.0), px(2.0), px(0.0), color)
    }

    /// Medium elevation shadow
    pub fn medium(color: Hsla) -> Self {
        Self::new(px(0.0), px(4.0), px(6.0), px(-1.0), color)
    }

    /// Large elevation shadow
    pub fn large(color: Hsla) -> Self {
        Self::new(px(0.0), px(10.0), px(15.0), px(-3.0), color)
    }

    /// Extra large elevation shadow
    pub fn xl(color: Hsla) -> Self {
        Self::new(px(0.0), px(20.0), px(25.0), px(-5.0), color)
    }
}

/// Animation/transition settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// Duration in milliseconds
    pub duration: u32,
    /// Easing function
    pub easing: EasingFunction,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            duration: 200,
            easing: EasingFunction::EaseInOut,
        }
    }
}

/// Easing function for animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing() {
        let spacing = Spacing::default();
        assert!(spacing.get("md").is_some());
        assert!(spacing.get("invalid").is_none());

        let uniform = spacing.uniform("md").unwrap();
        assert_eq!(uniform.top, uniform.bottom);
        assert_eq!(uniform.left, uniform.right);
    }

    #[test]
    fn test_border_radius() {
        let radius = BorderRadius::uniform(px(4.0));
        assert_eq!(radius.top_left, radius.bottom_right);
    }

    #[test]
    fn test_typography_defaults() {
        let typography = Typography::default();
        assert!(!typography.font_family.sans.is_empty());
        assert_eq!(typography.font_scale.base, 1.0);
    }
}
