//! Paragraph element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    combine_highlights, div, prelude::*, rems, App, ElementId, HighlightStyle, Hsla, ParentElement,
    SharedString, Styled, StyledText,
};

use super::super::inline_style::{InlinePalette, RichText};
use super::super::selectable_text::SelectableText;
use super::super::selection::MarkdownSelection;
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

/// Everything a text run needs to take part in document-wide selection:
/// the shared state, this run's document-order index, and the highlight
/// color a selected range paints with.
pub(crate) struct RunContext {
    pub selection: MarkdownSelection,
    pub run: usize,
    pub selection_background: Hsla,
}

/// One rich text run: a `StyledText` carrying the span highlights (plus the
/// selected range as a background highlight), wrapped in [`SelectableText`]
/// so it drags as part of the document and its link ranges open on click.
pub(crate) fn rich_text_run(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    palette: &InlinePalette,
    run_cx: RunContext,
) -> gpui::AnyElement {
    let (text, highlights) = rich_text.to_highlights_with(palette);
    let links = rich_text.link_ranges();
    let styled = selection_styled_text(text, highlights, &run_cx);

    let element = SelectableText::new(id, styled, run_cx.run, run_cx.selection);
    if links.is_empty() {
        return element.into_any_element();
    }
    let (ranges, urls): (Vec<_>, Vec<_>) = links.into_iter().unzip();
    element
        .on_click(ranges, move |range_ix, _window, cx| {
            if let Some(url) = urls.get(range_ix) {
                cx.open_url(url);
            }
        })
        .into_any_element()
}

/// A `StyledText` with the run's selected range merged in as one more
/// background highlight — which is all "rendering the selection" is.
pub(crate) fn selection_styled_text(
    text: String,
    highlights: Vec<(std::ops::Range<usize>, HighlightStyle)>,
    run_cx: &RunContext,
) -> StyledText {
    let highlights = match run_cx.selection.range_in_run(run_cx.run, text.len()) {
        Some(range) => combine_highlights(
            highlights,
            [(
                range,
                HighlightStyle {
                    background_color: Some(run_cx.selection_background),
                    ..Default::default()
                },
            )],
        )
        .collect(),
        None => highlights,
    };
    let text: SharedString = text.into();
    StyledText::new(text).with_highlights(highlights)
}

/// Render a paragraph element with rich text (bold, italic, strikethrough,
/// inline code, clickable links, and selection).
pub(crate) fn rich_paragraph(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    style: &TextStyle,
    palette: &InlinePalette,
    run_cx: RunContext,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(rich_text_run(id, rich_text, palette, run_cx))
}
