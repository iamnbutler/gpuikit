//! Paragraph element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, rems, App, ElementId, InteractiveText, ParentElement, SharedString, Styled,
    StyledText,
};

use super::super::inline_style::{InlinePalette, RichText};
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

/// One rich text run: a `StyledText` carrying the span highlights, wrapped in
/// `InteractiveText` when any span is a link so the link ranges are clickable
/// (opening in the default browser) without breaking the text into blocks.
pub(crate) fn rich_text_run(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    palette: &InlinePalette,
) -> gpui::AnyElement {
    let (text, highlights) = rich_text.to_highlights_with(palette);
    let styled: SharedString = text.into();
    let styled = StyledText::new(styled).with_highlights(highlights);

    let links = rich_text.link_ranges();
    if links.is_empty() {
        return styled.into_any_element();
    }
    let (ranges, urls): (Vec<_>, Vec<_>) = links.into_iter().unzip();
    InteractiveText::new(id, styled)
        .on_click(ranges, move |range_ix, _window, cx| {
            if let Some(url) = urls.get(range_ix) {
                cx.open_url(url);
            }
        })
        .into_any_element()
}

/// Render a paragraph element with rich text (bold, italic, strikethrough,
/// inline code, and clickable links).
pub fn rich_paragraph(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    style: &TextStyle,
    palette: &InlinePalette,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color)
        .child(rich_text_run(id, rich_text, palette))
}
