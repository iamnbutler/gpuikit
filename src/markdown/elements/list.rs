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

/// Whether a row draws the item's marker, or only reserves room for it.
///
/// An item can hold more than one block — a loose list wraps each of them in a
/// paragraph — and every block is its own row. Only the first of them is the
/// item: it draws the marker and announces itself as a list item. Each later
/// block lays the same marker out invisibly, so its text starts in the same
/// column, and announces itself as a paragraph rather than telling assistive
/// technology the list has more items than it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ItemMarker {
    /// An item's first block: paint the marker.
    Shown(String),
    /// A later block of the same item: reserve the marker's width, draw
    /// nothing.
    Hidden(String),
}

impl ItemMarker {
    /// The marker text — the same string either way; only whether it is
    /// painted differs.
    pub(crate) fn text(&self) -> &str {
        match self {
            ItemMarker::Shown(marker) | ItemMarker::Hidden(marker) => marker,
        }
    }

    /// How the row's text run announces itself.
    fn run_role(&self) -> RunRole {
        match self {
            ItemMarker::Shown(_) => RunRole::ListItem,
            ItemMarker::Hidden(_) => RunRole::Paragraph,
        }
    }
}

/// Render a list item element with rich text (bold, italic, strikethrough,
/// inline code, clickable links, and selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rich_list_item(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    marker: ItemMarker,
    indent_level: usize,
    style: &TextStyle,
    palette: &InlinePalette,
    run_cx: RunContext,
    cx: &App,
) -> impl IntoElement {
    let indent = rems(indent_level as f32 * 1.5);
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());
    let role = marker.run_role();
    let hidden = matches!(marker, ItemMarker::Hidden(_));
    let marker_text = marker.text().to_string();

    div()
        .flex()
        .flex_row()
        .pl(indent)
        .gap(rems(0.5))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(
            // Laid out either way: a hand-computed padding would have to guess
            // the glyph width of `•` against `10.`, and the text column of an
            // item's later blocks has to line up with its first.
            div()
                .flex_none()
                .when(hidden, |el| el.opacity(0.))
                .child(marker_text),
        )
        .child(
            // As in `list_item`: the flex item's automatic minimum size would
            // otherwise keep the text on one line.
            div()
                .flex_1()
                .min_w_0()
                .child(rich_text_run(id, rich_text, palette, role, run_cx)),
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
