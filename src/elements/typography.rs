//! Typography components for consistent text styling.
//!
//! Provides heading, paragraph, and text components with a consistent typography scale.
//!
//! # Examples
//!
//! ```
//! use gpuikit::elements::typography::*;
//!
//! // Headings
//! h1("Main Title")
//! h2("Section Title")
//! h3("Subsection")
//! h4("Minor Heading")
//!
//! // Paragraphs
//! p("Paragraph content...")
//!
//! // Text utilities with variants
//! text("Inline text").muted()
//! text("Warning").destructive()
//! text("Code").code()
//!
//! // Builder pattern options
//! h1("Title").align(TextAlign::Center).truncate(true)
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, rems, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
};

/// The typescale ratio (minor third).
const TYPESCALE_RATIO: f32 = 1.2;

/// Base font size in rems.
const BASE_SIZE: f32 = 1.0;

/// Text alignment options.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Color variants for text.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextVariant {
    #[default]
    Default,
    Muted,
    Destructive,
    Accent,
}

// ============================================================================
// Heading Components (H1-H4)
// ============================================================================

/// Creates an H1 heading element.
pub fn h1(content: impl Into<SharedString>) -> Heading {
    Heading::new(content, 1)
}

/// Creates an H2 heading element.
pub fn h2(content: impl Into<SharedString>) -> Heading {
    Heading::new(content, 2)
}

/// Creates an H3 heading element.
pub fn h3(content: impl Into<SharedString>) -> Heading {
    Heading::new(content, 3)
}

/// Creates an H4 heading element.
pub fn h4(content: impl Into<SharedString>) -> Heading {
    Heading::new(content, 4)
}

/// A heading element (h1-h4).
#[derive(IntoElement)]
pub struct Heading {
    content: SharedString,
    level: u8,
    align: TextAlign,
    truncate: bool,
    variant: TextVariant,
}

impl Heading {
    /// Creates a new heading at the specified level (1-4).
    pub fn new(content: impl Into<SharedString>, level: u8) -> Self {
        Self {
            content: content.into(),
            level: level.clamp(1, 4),
            align: TextAlign::default(),
            truncate: false,
            variant: TextVariant::default(),
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Enables text truncation with ellipsis.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the text variant.
    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = TextVariant::Muted;
        self
    }

    /// Uses destructive/danger color.
    pub fn destructive(mut self) -> Self {
        self.variant = TextVariant::Destructive;
        self
    }

    /// Uses accent color.
    pub fn accent(mut self) -> Self {
        self.variant = TextVariant::Accent;
        self
    }

    fn font_size(&self) -> f32 {
        let scale_power = match self.level {
            1 => 4,
            2 => 3,
            3 => 2,
            4 => 1,
            _ => 0,
        };
        BASE_SIZE * TYPESCALE_RATIO.powi(scale_power)
    }
}

impl RenderOnce for Heading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = self.font_size();

        let text_color = match self.variant {
            TextVariant::Default => theme.fg(),
            TextVariant::Muted => theme.fg_muted(),
            TextVariant::Destructive => theme.danger(),
            TextVariant::Accent => theme.accent(),
        };

        let mut el = div()
            .w_full()
            .text_size(rems(size))
            .line_height(rems(size * 1.2))
            .font_weight(FontWeight::BOLD)
            .text_color(text_color);

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        el.child(self.content)
    }
}

// ============================================================================
// Paragraph Component
// ============================================================================

/// Creates a paragraph element.
pub fn p(content: impl Into<SharedString>) -> Paragraph {
    Paragraph::new(content)
}

/// A paragraph element for body text.
#[derive(IntoElement)]
pub struct Paragraph {
    content: SharedString,
    align: TextAlign,
    truncate: bool,
    variant: TextVariant,
}

impl Paragraph {
    /// Creates a new paragraph.
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            align: TextAlign::default(),
            truncate: false,
            variant: TextVariant::default(),
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Enables text truncation with ellipsis.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the text variant.
    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = TextVariant::Muted;
        self
    }

    /// Uses destructive/danger color.
    pub fn destructive(mut self) -> Self {
        self.variant = TextVariant::Destructive;
        self
    }

    /// Uses accent color.
    pub fn accent(mut self) -> Self {
        self.variant = TextVariant::Accent;
        self
    }
}

impl RenderOnce for Paragraph {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let text_color = match self.variant {
            TextVariant::Default => theme.fg(),
            TextVariant::Muted => theme.fg_muted(),
            TextVariant::Destructive => theme.danger(),
            TextVariant::Accent => theme.accent(),
        };

        let mut el = div()
            .w_full()
            .text_size(rems(BASE_SIZE))
            .line_height(rems(BASE_SIZE * 1.5))
            .text_color(text_color);

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        el.child(self.content)
    }
}

// ============================================================================
// Text Component
// ============================================================================

/// Creates a generic styled text element.
pub fn text(content: impl Into<SharedString>) -> Text {
    Text::new(content)
}

/// A generic styled text element with various options.
#[derive(IntoElement)]
pub struct Text {
    content: SharedString,
    align: TextAlign,
    truncate: bool,
    variant: TextVariant,
    code: bool,
    small: bool,
    bold: bool,
}

impl Text {
    /// Creates a new text element.
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            align: TextAlign::default(),
            truncate: false,
            variant: TextVariant::default(),
            code: false,
            small: false,
            bold: false,
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Enables text truncation with ellipsis.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the text variant.
    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = TextVariant::Muted;
        self
    }

    /// Uses destructive/danger color.
    pub fn destructive(mut self) -> Self {
        self.variant = TextVariant::Destructive;
        self
    }

    /// Uses accent color.
    pub fn accent(mut self) -> Self {
        self.variant = TextVariant::Accent;
        self
    }

    /// Applies code/monospace styling.
    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }

    /// Uses a smaller text size.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// Uses bold font weight.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let text_color = match self.variant {
            TextVariant::Default => theme.fg(),
            TextVariant::Muted => theme.fg_muted(),
            TextVariant::Destructive => theme.danger(),
            TextVariant::Accent => theme.accent(),
        };

        let size = if self.small {
            BASE_SIZE * 0.875
        } else if self.code {
            BASE_SIZE * 0.875
        } else {
            BASE_SIZE
        };

        let mut el = div()
            .text_size(rems(size))
            .line_height(rems(size * 1.5))
            .text_color(text_color);

        if self.bold {
            el = el.font_weight(FontWeight::BOLD);
        }

        if self.code {
            el = el.font_family("monospace");
        }

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        el.child(self.content)
    }
}

// ============================================================================
// Optional: Lead and Small Components
// ============================================================================

/// Creates a lead text element (larger intro text).
pub fn lead(content: impl Into<SharedString>) -> Lead {
    Lead::new(content)
}

/// A lead text element, typically used for introductory paragraphs.
#[derive(IntoElement)]
pub struct Lead {
    content: SharedString,
    align: TextAlign,
    truncate: bool,
    variant: TextVariant,
}

impl Lead {
    /// Creates a new lead text element.
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            align: TextAlign::default(),
            truncate: false,
            variant: TextVariant::default(),
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Enables text truncation with ellipsis.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the text variant.
    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = TextVariant::Muted;
        self
    }
}

impl RenderOnce for Lead {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = BASE_SIZE * TYPESCALE_RATIO; // One step larger than body

        let text_color = match self.variant {
            TextVariant::Default => theme.fg(),
            TextVariant::Muted => theme.fg_muted(),
            TextVariant::Destructive => theme.danger(),
            TextVariant::Accent => theme.accent(),
        };

        let mut el = div()
            .w_full()
            .text_size(rems(size))
            .line_height(rems(size * 1.5))
            .text_color(text_color);

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        el.child(self.content)
    }
}

/// Creates a small text element.
pub fn small(content: impl Into<SharedString>) -> Small {
    Small::new(content)
}

/// A small text element for captions, labels, etc.
#[derive(IntoElement)]
pub struct Small {
    content: SharedString,
    align: TextAlign,
    truncate: bool,
    variant: TextVariant,
}

impl Small {
    /// Creates a new small text element.
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            align: TextAlign::default(),
            truncate: false,
            variant: TextVariant::Muted, // Default to muted for small text
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Enables text truncation with ellipsis.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the text variant.
    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses default color.
    pub fn default_color(mut self) -> Self {
        self.variant = TextVariant::Default;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = TextVariant::Muted;
        self
    }
}

impl RenderOnce for Small {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = BASE_SIZE * 0.875; // Smaller than body

        let text_color = match self.variant {
            TextVariant::Default => theme.fg(),
            TextVariant::Muted => theme.fg_muted(),
            TextVariant::Destructive => theme.danger(),
            TextVariant::Accent => theme.accent(),
        };

        let mut el = div()
            .text_size(rems(size))
            .line_height(rems(size * 1.5))
            .text_color(text_color);

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        el.child(self.content)
    }
}

// ============================================================================
// Blockquote Component
// ============================================================================

/// Creates a blockquote element.
pub fn blockquote(content: impl Into<SharedString>) -> Blockquote {
    Blockquote::new(content)
}

/// A blockquote element for quoted content.
#[derive(IntoElement)]
pub struct Blockquote {
    content: SharedString,
    align: TextAlign,
}

impl Blockquote {
    /// Creates a new blockquote element.
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            align: TextAlign::default(),
        }
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl RenderOnce for Blockquote {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut el = div()
            .w_full()
            .pl(rems(1.0))
            .border_l_2()
            .border_color(theme.border())
            .text_size(rems(BASE_SIZE))
            .line_height(rems(BASE_SIZE * 1.5))
            .text_color(theme.fg_muted())
            .italic();

        el = match self.align {
            TextAlign::Left => el,
            TextAlign::Center => el.text_center(),
            TextAlign::Right => el.text_right(),
        };

        el.child(self.content)
    }
}
