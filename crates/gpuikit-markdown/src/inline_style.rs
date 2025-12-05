//! Inline text styling for markdown rendering.
//!
//! This module provides types for tracking and rendering inline text styles
//! like bold, italic, and strikethrough within markdown text.

use gpui::{FontStyle, FontWeight, HighlightStyle, StrikethroughStyle};
use std::ops::Range;

/// Inline text style flags that can be combined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
}

impl InlineStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn to_highlight_style(self) -> HighlightStyle {
        let mut style = HighlightStyle::default();

        if self.bold {
            style.font_weight = Some(FontWeight::BOLD);
        }

        if self.italic {
            style.font_style = Some(FontStyle::Italic);
        }

        if self.strikethrough {
            style.strikethrough = Some(StrikethroughStyle {
                thickness: gpui::px(1.0),
                ..Default::default()
            });
        }

        style
    }

    pub fn is_empty(&self) -> bool {
        !self.bold && !self.italic && !self.strikethrough
    }
}

/// A span of text with associated styling.
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub style: InlineStyle,
}

/// Rich text container that holds styled text spans.
#[derive(Clone, Debug, Default)]
pub struct RichText {
    spans: Vec<TextSpan>,
}

impl RichText {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, text: impl Into<String>, style: InlineStyle) {
        let text = text.into();
        if text.is_empty() {
            return;
        }

        if let Some(last) = self.spans.last_mut() {
            if last.style == style {
                last.text.push_str(&text);
                return;
            }
        }

        self.spans.push(TextSpan { text, style });
    }

    pub fn push_plain(&mut self, text: impl Into<String>) {
        self.push(text, InlineStyle::default());
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.text.is_empty())
    }

    pub fn clear(&mut self) {
        self.spans.clear();
    }

    pub fn to_plain_text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn to_highlights(&self) -> (String, Vec<(Range<usize>, HighlightStyle)>) {
        let mut text = String::new();
        let mut highlights = Vec::new();

        for span in &self.spans {
            let start = text.len();
            text.push_str(&span.text);
            let end = text.len();

            if !span.style.is_empty() {
                highlights.push((start..end, span.style.to_highlight_style()));
            }
        }

        (text, highlights)
    }

    pub fn spans(&self) -> &[TextSpan] {
        &self.spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_style_default() {
        let style = InlineStyle::new();
        assert!(!style.bold);
        assert!(!style.italic);
        assert!(!style.strikethrough);
        assert!(style.is_empty());
    }

    #[test]
    fn test_inline_style_builders() {
        let bold = InlineStyle::new().with_bold();
        assert!(bold.bold);
        assert!(!bold.italic);
        assert!(!bold.is_empty());

        let italic = InlineStyle::new().with_italic();
        assert!(italic.italic);
        assert!(!italic.bold);

        let strike = InlineStyle::new().with_strikethrough();
        assert!(strike.strikethrough);

        let combined = InlineStyle::new()
            .with_bold()
            .with_italic()
            .with_strikethrough();
        assert!(combined.bold);
        assert!(combined.italic);
        assert!(combined.strikethrough);
    }

    #[test]
    fn test_inline_style_to_highlight() {
        let bold = InlineStyle::new().with_bold();
        let highlight = bold.to_highlight_style();
        assert_eq!(highlight.font_weight, Some(FontWeight::BOLD));
        assert_eq!(highlight.font_style, None);

        let italic = InlineStyle::new().with_italic();
        let highlight = italic.to_highlight_style();
        assert_eq!(highlight.font_style, Some(FontStyle::Italic));
        assert_eq!(highlight.font_weight, None);

        let strike = InlineStyle::new().with_strikethrough();
        let highlight = strike.to_highlight_style();
        assert!(highlight.strikethrough.is_some());
    }

    #[test]
    fn test_rich_text_empty() {
        let rt = RichText::new();
        assert!(rt.is_empty());
        assert_eq!(rt.to_plain_text(), "");
    }

    #[test]
    fn test_rich_text_push_plain() {
        let mut rt = RichText::new();
        rt.push_plain("Hello ");
        rt.push_plain("World");
        assert_eq!(rt.to_plain_text(), "Hello World");
    }

    #[test]
    fn test_rich_text_merge_same_style() {
        let mut rt = RichText::new();
        let bold = InlineStyle::new().with_bold();
        rt.push("Hello ", bold);
        rt.push("World", bold);
        assert_eq!(rt.spans().len(), 1);
        assert_eq!(rt.to_plain_text(), "Hello World");
    }

    #[test]
    fn test_rich_text_different_styles() {
        let mut rt = RichText::new();
        let bold = InlineStyle::new().with_bold();
        let italic = InlineStyle::new().with_italic();
        rt.push("Bold", bold);
        rt.push("Italic", italic);
        assert_eq!(rt.spans().len(), 2);
        assert_eq!(rt.to_plain_text(), "BoldItalic");
    }

    #[test]
    fn test_rich_text_highlights() {
        let mut rt = RichText::new();
        let bold = InlineStyle::new().with_bold();
        rt.push_plain("Hello ");
        rt.push("bold", bold);
        rt.push_plain(" world");

        let (text, highlights) = rt.to_highlights();
        assert_eq!(text, "Hello bold world");
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].0, 6..10);
        assert_eq!(highlights[0].1.font_weight, Some(FontWeight::BOLD));
    }

    #[test]
    fn test_rich_text_skip_empty() {
        let mut rt = RichText::new();
        rt.push_plain("");
        rt.push_plain("Text");
        rt.push_plain("");
        assert_eq!(rt.spans().len(), 1);
    }
}
