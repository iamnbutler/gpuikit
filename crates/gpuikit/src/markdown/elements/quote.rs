//! Block quote element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, px, rems, App, ParentElement, SharedString, Styled, StyledText};

use super::super::inline_style::RichText;
use super::super::style::TextStyle;

/// Render a block quote element with plain text.
pub fn block_quote(
    text: impl Into<String>,
    style: &TextStyle,
    border_color: Option<gpui::Hsla>,
    text_color: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let text: String = text.into();
    let theme = cx.theme();

    div()
        .w_full()
        .pl(rems(1.0))
        .border_l(px(3.0))
        .border_color(border_color.unwrap_or(theme.border()))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color.unwrap_or(theme.fg_muted()))
        .italic()
        .child(text)
}

/// Render a block quote element with rich text (supporting bold, italic, strikethrough).
pub fn rich_block_quote(
    rich_text: &RichText,
    style: &TextStyle,
    border_color: Option<gpui::Hsla>,
    text_color: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let (text, highlights) = rich_text.to_highlights();

    let styled_text: SharedString = text.into();

    div()
        .w_full()
        .pl(rems(1.0))
        .border_l(px(3.0))
        .border_color(border_color.unwrap_or(theme.border()))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color.unwrap_or(theme.fg_muted()))
        .italic()
        .child(StyledText::new(styled_text).with_highlights(highlights))
}
