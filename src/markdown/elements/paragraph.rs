//! Paragraph element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ParentElement, SharedString, Styled, StyledText};

use super::super::inline_style::RichText;
use super::super::style::TextStyle;

/// Render a paragraph element with plain text.
pub fn paragraph(text: impl Into<String>, style: &TextStyle, cx: &App) -> impl IntoElement {
    let text: String = text.into();
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(text)
}

/// Render a paragraph element with rich text (supporting bold, italic, strikethrough).
pub fn rich_paragraph(rich_text: &RichText, style: &TextStyle, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());
    let (text, highlights) = rich_text.to_highlights();

    let styled_text: SharedString = text.into();

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(StyledText::new(styled_text).with_highlights(highlights))
}
