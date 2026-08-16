//! List elements for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ElementId, ParentElement, Styled};

use super::super::inline_style::{InlinePalette, RichText};
use super::super::selectable_text::RunRole;
use super::super::style::TextStyle;
use super::paragraph::{rich_text_run, RunContext};

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
        // Without `min_w_0` the text is a flex item with an automatic minimum
        // size of one unbroken line, so a long item runs off the edge instead
        // of wrapping.
        .child(div().flex_1().min_w_0().child(text))
}

/// Render a list item element with rich text (bold, italic, strikethrough,
/// inline code, clickable links, and selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rich_list_item(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    marker: String,
    indent_level: usize,
    style: &TextStyle,
    palette: &InlinePalette,
    run_cx: RunContext,
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
        .child(
            // As in `list_item`: the flex item's automatic minimum size would
            // otherwise keep the text on one line.
            div().flex_1().min_w_0().child(rich_text_run(
                id,
                rich_text,
                palette,
                RunRole::ListItem,
                run_cx,
            )),
        )
}

/// Get the marker for an unordered list item.
pub fn unordered_marker() -> String {
    "•".to_string()
}

/// Get the marker for an ordered list item.
pub fn ordered_marker(index: u64) -> String {
    format!("{}.", index)
}
