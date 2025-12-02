//! Heading element for markdown.

use gpui::{div, prelude::*, rems, App, ParentElement, Styled};
use gpuikit_theme::ActiveTheme;

use crate::style::TextStyle;

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

/// Render a heading element.
pub fn heading(text: String, style: &TextStyle, cx: &App) -> impl IntoElement {
    div()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_weight(style.weight)
        .text_color(style.color.unwrap_or(cx.theme().fg))
        .mt(rems(style.margin_top))
        .child(text)
}
