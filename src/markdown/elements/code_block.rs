//! Code block element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{App, ElementId, ParentElement, SharedString, Styled, div, prelude::*, rems};

use super::super::code_highlight::code_highlights;
use super::super::selectable_text::{RunRole, SelectableText};
use super::super::style::TextStyle;
use super::paragraph::{RunContext, selection_styled_text};

/// Render a code block element. The text inside is a selectable run — code
/// is the thing people copy most.
///
/// `language` is the fence's normalized language token. It buys syntax
/// highlighting only when the `editor` feature is on *and* the app called
/// [`init_code_highlighting`](super::super::code_highlight); otherwise, and
/// for a language syntect has no grammar for, the block renders plain
/// monospace exactly as before.
///
/// The renderer also passes `None` for a block whose fence has not closed yet,
/// which is why a block gains its colors when it finishes streaming: a growing
/// block misses the highlight cache on every delta and evicts everything else,
/// so it takes this same no-syntect path until it settles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn code_block(
    id: impl Into<ElementId>,
    text: String,
    language: Option<&str>,
    style: &TextStyle,
    font_family: &SharedString,
    bg: Option<gpui::Hsla>,
    border: Option<gpui::Hsla>,
    run_cx: RunContext,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    // Resolved once, and handed to the highlighter as well as to the div, so
    // the light/dark decision follows the surface the highlights are painted
    // on rather than the window at large.
    let background = bg.unwrap_or(theme.surface());

    let highlights = code_highlights(&text, language, background, cx);
    let (styled, plain) = selection_styled_text(text, highlights, &run_cx);

    div()
        .px(rems(1.0))
        .py(rems(0.75))
        .rounded(rems(0.375))
        .bg(background)
        .border_1()
        .border_color(border.unwrap_or(theme.border()))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_family(font_family.clone())
        .text_color(style.color.unwrap_or(theme.fg()))
        .overflow_hidden()
        .child(SelectableText::new(
            id,
            styled,
            plain,
            RunRole::Code,
            run_cx.run,
            run_cx.selection,
        ))
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
