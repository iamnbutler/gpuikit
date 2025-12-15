//! Code block element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ParentElement, SharedString, Styled};

use super::super::style::TextStyle;

/// Render a code block element.
///
/// TODO: Replace with gpuikit-editor readonly view for syntax highlighting.
pub fn code_block(
    text: String,
    _language: Option<&str>,
    style: &TextStyle,
    font_family: &SharedString,
    bg: Option<gpui::Hsla>,
    border: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .px(rems(1.0))
        .py(rems(0.75))
        .rounded(rems(0.375))
        .bg(bg.unwrap_or(theme.surface()))
        .border_1()
        .border_color(border.unwrap_or(theme.border()))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_family(font_family.clone())
        .text_color(style.color.unwrap_or(theme.fg()))
        .overflow_hidden()
        .child(text)
}

/// Render inline code.
pub fn inline_code(
    text: String,
    font_family: &SharedString,
    bg: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .px(rems(0.25))
        .rounded(rems(0.25))
        .bg(bg.unwrap_or(theme.surface()))
        .font_family(font_family.clone())
        .text_size(rems(0.875))
        .child(text)
}
