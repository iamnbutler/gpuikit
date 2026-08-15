//! Inline text styling for markdown rendering.
//!
//! This module provides types for tracking and rendering inline text styles
//! like bold, italic, strikethrough, inline code, and links within markdown
//! text. Code and links stay *inline* — spans within one text run — rather
//! than being flushed as separate block elements, so a sentence survives
//! having a link or a code chip in the middle of it.

use gpui::{FontStyle, FontWeight, HighlightStyle, Hsla, SharedString, StrikethroughStyle};
use std::ops::Range;

/// Inline text style flags that can be combined.
///
/// `link` is an index into the owning [`RichText`]'s URL table rather than
/// the URL itself, which keeps this type `Copy` and keeps span merging
/// correct: two adjacent links to different URLs have different indices, so
/// they never merge into one clickable range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link: Option<u32>,
}

/// Theme-resolved colors for the inline styles that need them. Resolved by
/// the element (which has the theme) and passed into
/// [`RichText::to_highlights_with`]; `None` leaves that aspect unstyled.
#[derive(Clone, Copy, Debug, Default)]
pub struct InlinePalette {
    /// Background wash behind inline code spans.
    pub code_background: Option<Hsla>,
    /// Text color for link spans (they are also underlined).
    pub link_color: Option<Hsla>,
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

    pub fn with_code(mut self) -> Self {
        self.code = true;
        self
    }

    pub fn to_highlight_style(self) -> HighlightStyle {
        self.to_highlight_style_with(&InlinePalette::default())
    }

    pub fn to_highlight_style_with(self, palette: &InlinePalette) -> HighlightStyle {
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

        if self.code {
            style.background_color = palette.code_background;
        }

        if self.link.is_some() {
            style.color = palette.link_color;
            style.underline = Some(gpui::UnderlineStyle {
                thickness: gpui::px(1.0),
                color: palette.link_color,
                wavy: false,
            });
        }

        style
    }

    pub fn is_empty(&self) -> bool {
        !self.bold && !self.italic && !self.strikethrough && !self.code && self.link.is_none()
    }
}

/// A span of text with associated styling.
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub style: InlineStyle,
}

/// Rich text container that holds styled text spans, plus the URL table that
/// link spans index into.
#[derive(Clone, Debug, Default)]
pub struct RichText {
    spans: Vec<TextSpan>,
    links: Vec<SharedString>,
}

impl RichText {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a link destination, returning the index for
    /// [`InlineStyle::link`] on the spans that carry its text.
    pub fn add_link(&mut self, url: impl Into<SharedString>) -> u32 {
        self.links.push(url.into());
        (self.links.len() - 1) as u32
    }

    /// The registered link destinations, in registration order.
    pub fn links(&self) -> &[SharedString] {
        &self.links
    }

    /// Byte ranges of the flattened text that are links, with their
    /// destinations — the input for an interactive text element's clickable
    /// ranges. Adjacent spans of the same link merge into one range.
    pub fn link_ranges(&self) -> Vec<(Range<usize>, SharedString)> {
        let mut ranges: Vec<(Range<usize>, u32)> = Vec::new();
        let mut offset = 0;
        for span in &self.spans {
            let end = offset + span.text.len();
            if let Some(link) = span.style.link {
                match ranges.last_mut() {
                    Some((range, last)) if *last == link && range.end == offset => {
                        range.end = end;
                    }
                    _ => ranges.push((offset..end, link)),
                }
            }
            offset = end;
        }
        ranges
            .into_iter()
            .filter_map(|(range, ix)| Some((range, self.links.get(ix as usize)?.clone())))
            .collect()
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
        self.links.clear();
    }

    pub fn to_plain_text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn to_highlights(&self) -> (String, Vec<(Range<usize>, HighlightStyle)>) {
        self.to_highlights_with(&InlinePalette::default())
    }

    pub fn to_highlights_with(
        &self,
        palette: &InlinePalette,
    ) -> (String, Vec<(Range<usize>, HighlightStyle)>) {
        let mut text = String::new();
        let mut highlights = Vec::new();

        for span in &self.spans {
            let start = text.len();
            text.push_str(&span.text);
            let end = text.len();

            if !span.style.is_empty() {
                highlights.push((start..end, span.style.to_highlight_style_with(palette)));
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

    #[test]
    fn test_code_span_styles_without_backticks() {
        let mut rt = RichText::new();
        rt.push_plain("run ");
        rt.push("cargo test", InlineStyle::new().with_code());
        rt.push_plain(" first");

        let palette = InlinePalette {
            code_background: Some(gpui::hsla(0., 0., 0.5, 1.)),
            link_color: None,
        };
        let (text, highlights) = rt.to_highlights_with(&palette);
        assert_eq!(text, "run cargo test first");
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].0, 4..14);
        assert_eq!(highlights[0].1.background_color, palette.code_background);
    }

    #[test]
    fn test_link_spans_stay_inline_and_click_ranges_resolve() {
        let mut rt = RichText::new();
        rt.push_plain("see ");
        let a = rt.add_link("https://a.example");
        rt.push(
            "first",
            InlineStyle {
                link: Some(a),
                ..Default::default()
            },
        );
        rt.push_plain(" and ");
        let b = rt.add_link("https://b.example");
        rt.push(
            "second",
            InlineStyle {
                link: Some(b),
                ..Default::default()
            },
        );

        // The sentence survives as one text run…
        let (text, _) = rt.to_highlights();
        assert_eq!(text, "see first and second");

        // …with two distinct clickable ranges pointing at their own URLs.
        let ranges = rt.link_ranges();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].0, 4..9);
        assert_eq!(ranges[0].1.as_ref(), "https://a.example");
        assert_eq!(ranges[1].0, 14..20);
        assert_eq!(ranges[1].1.as_ref(), "https://b.example");
    }

    #[test]
    fn test_adjacent_spans_of_different_links_do_not_merge() {
        let mut rt = RichText::new();
        let a = rt.add_link("https://a.example");
        let b = rt.add_link("https://b.example");
        rt.push(
            "one",
            InlineStyle {
                link: Some(a),
                ..Default::default()
            },
        );
        rt.push(
            "two",
            InlineStyle {
                link: Some(b),
                ..Default::default()
            },
        );
        assert_eq!(rt.spans().len(), 2);
        assert_eq!(rt.link_ranges().len(), 2);
    }

    #[test]
    fn test_split_link_spans_merge_into_one_range() {
        // A link whose text is interrupted by emphasis is still one link.
        let mut rt = RichText::new();
        let a = rt.add_link("https://a.example");
        let plain = InlineStyle {
            link: Some(a),
            ..Default::default()
        };
        let bold = InlineStyle {
            link: Some(a),
            bold: true,
            ..Default::default()
        };
        rt.push("very ", plain);
        rt.push("bold", bold);
        rt.push(" link", plain);

        let ranges = rt.link_ranges();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].0, 0..14);
    }

    #[test]
    fn test_link_styling_applies_color_and_underline() {
        let mut rt = RichText::new();
        let a = rt.add_link("https://a.example");
        rt.push(
            "link",
            InlineStyle {
                link: Some(a),
                ..Default::default()
            },
        );

        let palette = InlinePalette {
            code_background: None,
            link_color: Some(gpui::hsla(0.6, 0.8, 0.5, 1.)),
        };
        let (_, highlights) = rt.to_highlights_with(&palette);
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].1.color, palette.link_color);
        assert!(highlights[0].1.underline.is_some());
    }

    #[test]
    fn test_clear_drops_links_too() {
        let mut rt = RichText::new();
        let a = rt.add_link("https://a.example");
        rt.push(
            "x",
            InlineStyle {
                link: Some(a),
                ..Default::default()
            },
        );
        rt.clear();
        assert!(rt.is_empty());
        assert!(rt.links().is_empty());
        assert!(rt.link_ranges().is_empty());
    }
}
