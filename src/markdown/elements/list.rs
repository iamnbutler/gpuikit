//! List elements for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ElementId, ParentElement, Styled};

use super::super::inline_style::{InlinePalette, RichText};
use super::super::style::TextStyle;
use super::paragraph::rich_text_run;

/// Render a list item element with plain text.
pub fn list_item(
    text: impl Into<String>,
    marker: String,
    indent_level: usize,
    style: &TextStyle,
    cx: &App,
) -> impl IntoElement {
    let text: String = text.into();
    let indent = rems(indent_level as f32 * 1.5);
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .flex()
        .flex_row()
        .pl(indent)
        .gap(rems(0.5))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(div().flex_none().child(marker))
        .child(div().flex_1().child(text))
}

/// Render a list item element with rich text (bold, italic, strikethrough,
/// inline code, and clickable links).
pub fn rich_list_item(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    marker: String,
    indent_level: usize,
    style: &TextStyle,
    palette: &InlinePalette,
    cx: &App,
) -> impl IntoElement {
    let indent = rems(indent_level as f32 * 1.5);
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .flex()
        .flex_row()
        .pl(indent)
        .gap(rems(0.5))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(div().flex_none().child(marker))
        .child(div().flex_1().child(rich_text_run(id, rich_text, palette)))
}

/// Get the marker for an unordered list item.
pub fn unordered_marker() -> String {
    "•".to_string()
}

/// Get the marker for an ordered list item.
pub fn ordered_marker(index: u64) -> String {
    format!("{}.", index)
}
