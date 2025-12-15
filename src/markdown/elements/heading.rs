//! Heading element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ParentElement, SharedString, Styled, StyledText};

use super::super::inline_style::RichText;
use super::super::style::TextStyle;

/// Heading level (h1-h6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(level: pulldown_cmark::HeadingLevel) -> Self {
        match level {
            pulldown_cmark::HeadingLevel::H1 => HeadingLevel::H1,
            pulldown_cmark::HeadingLevel::H2 => HeadingLevel::H2,
            pulldown_cmark::HeadingLevel::H3 => HeadingLevel::H3,
            pulldown_cmark::HeadingLevel::H4 => HeadingLevel::H4,
            pulldown_cmark::HeadingLevel::H5 => HeadingLevel::H5,
            pulldown_cmark::HeadingLevel::H6 => HeadingLevel::H6,
        }
    }
}

/// Render a heading element with plain text.
pub fn heading(text: impl Into<String>, style: &TextStyle, cx: &App) -> impl IntoElement {
    let text: String = text.into();
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_weight(style.weight)
        .text_color(text_color)
        .mt(rems(style.margin_top))
        .child(text)
}

/// Render a heading element with rich text (supporting bold, italic, strikethrough).
pub fn rich_heading(rich_text: &RichText, style: &TextStyle, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());
    let (text, highlights) = rich_text.to_highlights();

    let styled_text: SharedString = text.into();

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_weight(style.weight)
        .text_color(text_color)
        .mt(rems(style.margin_top))
        .child(StyledText::new(styled_text).with_highlights(highlights))
}
