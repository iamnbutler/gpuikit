//! Typography and styling for markdown rendering.
//!
//! Uses a perfect fourth (1.333) typescale for harmonious visual hierarchy.

use gpui::{FontWeight, Hsla, SharedString};

/// The typescale ratio (perfect fourth).
pub const TYPESCALE_RATIO: f32 = 1.333;

/// Base font size in rems.
pub const BASE_SIZE: f32 = 1.0;

/// Style configuration for a text element.
#[derive(Clone, Debug)]
pub struct TextStyle {
    /// Font size in rems.
    pub size: f32,
    /// Line height multiplier (relative to font size).
    pub line_height: f32,
    /// Font weight.
    pub weight: FontWeight,
    /// Text color (None = use theme default).
    pub color: Option<Hsla>,
    /// Top margin in rems.
    pub margin_top: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: BASE_SIZE,
            line_height: 1.5,
            weight: FontWeight::NORMAL,
            color: None,
            margin_top: 0.0,
        }
    }
}

impl TextStyle {
    /// Create a heading style at the given scale level.
    ///
    /// Level 1 is the largest (h1), level 6 is the smallest (h6).
    pub fn heading(level: u8) -> Self {
        let scale_power = match level {
            1 => 5,
            2 => 4,
            3 => 3,
            4 => 2,
            5 => 1,
            _ => 0,
        };

        let size = BASE_SIZE * TYPESCALE_RATIO.powi(scale_power);

        Self {
            size,
            line_height: 1.25,
            weight: FontWeight::BOLD,
            color: None,
            margin_top: size * 0.75,
        }
    }

    /// Create body text style.
    pub fn body() -> Self {
        Self {
            size: BASE_SIZE,
            line_height: 1.6,
            weight: FontWeight::NORMAL,
            color: None,
            margin_top: 0.0,
        }
    }

    /// Create code/monospace text style.
    pub fn code() -> Self {
        Self {
            size: BASE_SIZE * 0.875,
            line_height: 1.5,
            weight: FontWeight::NORMAL,
            color: None,
            margin_top: 0.0,
        }
    }
}

/// Complete style configuration for markdown rendering.
#[derive(Clone, Debug)]
pub struct MarkdownStyle {
    // Typography
    /// Body text style.
    pub body: TextStyle,
    /// H1 heading style.
    pub h1: TextStyle,
    /// H2 heading style.
    pub h2: TextStyle,
    /// H3 heading style.
    pub h3: TextStyle,
    /// H4 heading style.
    pub h4: TextStyle,
    /// H5 heading style.
    pub h5: TextStyle,
    /// H6 heading style.
    pub h6: TextStyle,
    /// Code/monospace text style.
    pub code: TextStyle,

    // Font families
    /// Font family for code blocks and inline code.
    pub code_font_family: SharedString,

    // Spacing
    /// Vertical spacing between block elements in rems.
    pub block_spacing: f32,

    // Colors (None = use theme defaults)
    /// Code block background color.
    pub code_block_bg: Option<Hsla>,
    /// Code block border color.
    pub code_block_border: Option<Hsla>,
    /// Inline code background color.
    pub inline_code_bg: Option<Hsla>,
    /// Block quote border color.
    pub block_quote_border: Option<Hsla>,
    /// Block quote text color.
    pub block_quote_text: Option<Hsla>,
    /// Horizontal rule color.
    pub rule_color: Option<Hsla>,
    /// Link text color.
    pub link_color: Option<Hsla>,
}

impl Default for MarkdownStyle {
    fn default() -> Self {
        Self {
            body: TextStyle::body(),
            h1: TextStyle::heading(1),
            h2: TextStyle::heading(2),
            h3: TextStyle::heading(3),
            h4: TextStyle::heading(4),
            h5: TextStyle::heading(5),
            h6: TextStyle::heading(6),
            code: TextStyle::code(),

            code_font_family: SharedString::from("monospace"),

            block_spacing: 1.0,

            code_block_bg: None,
            code_block_border: None,
            inline_code_bg: None,
            block_quote_border: None,
            block_quote_text: None,
            rule_color: None,
            link_color: None,
        }
    }
}

impl MarkdownStyle {
    /// Create a new style with default typography.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the code font family.
    pub fn code_font(mut self, family: impl Into<SharedString>) -> Self {
        self.code_font_family = family.into();
        self
    }

    /// Set the block spacing.
    pub fn block_spacing(mut self, spacing: f32) -> Self {
        self.block_spacing = spacing;
        self
    }

    /// Set all code-related colors.
    pub fn code_colors(mut self, bg: Hsla, border: Hsla) -> Self {
        self.code_block_bg = Some(bg);
        self.code_block_border = Some(border);
        self.inline_code_bg = Some(bg);
        self
    }

    /// Set block quote colors.
    pub fn block_quote_colors(mut self, border: Hsla, text: Hsla) -> Self {
        self.block_quote_border = Some(border);
        self.block_quote_text = Some(text);
        self
    }

    /// Set the horizontal rule color.
    pub fn rule_color(mut self, color: Hsla) -> Self {
        self.rule_color = Some(color);
        self
    }

    /// Set the link color.
    pub fn link_color(mut self, color: Hsla) -> Self {
        self.link_color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescale() {
        let h1 = TextStyle::heading(1);
        let h2 = TextStyle::heading(2);
        let body = TextStyle::body();

        assert!(h1.size > h2.size);
        assert!(h2.size > body.size);

        let ratio = h1.size / h2.size;
        assert!((ratio - TYPESCALE_RATIO).abs() < 0.01);
    }

    #[test]
    fn test_line_heights() {
        let body = TextStyle::body();
        let h1 = TextStyle::heading(1);

        assert!(body.line_height > h1.line_height);
    }
}
