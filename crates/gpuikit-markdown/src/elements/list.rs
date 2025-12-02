//! List elements for markdown.

use gpui::{div, prelude::*, rems, App, ParentElement, Styled};
use gpuikit_theme::ActiveTheme;

use crate::style::TextStyle;

/// Render a list item element.
pub fn list_item(
    text: String,
    marker: String,
    indent_level: usize,
    style: &TextStyle,
    cx: &App,
) -> impl IntoElement {
    let indent = rems(indent_level as f32 * 1.5);

    div()
        .flex()
        .flex_row()
        .pl(indent)
        .gap(rems(0.5))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(style.color.unwrap_or(cx.theme().fg))
        .child(div().flex_none().child(marker))
        .child(div().flex_1().child(text))
}

/// Get the marker for an unordered list item.
pub fn unordered_marker() -> String {
    "•".to_string()
}

/// Get the marker for an ordered list item.
pub fn ordered_marker(index: u64) -> String {
    format!("{}.", index)
}
