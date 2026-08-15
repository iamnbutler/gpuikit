//! Code block element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ElementId, ParentElement, SharedString, Styled};

use super::super::selectable_text::SelectableText;
use super::super::style::TextStyle;
use super::paragraph::{selection_styled_text, RunContext};

/// Render a code block element. The text inside is a selectable run — code
/// is the thing people copy most.
///
/// TODO: Replace with gpuikit-editor readonly view for syntax highlighting.
#[allow(clippy::too_many_arguments)]
pub(crate) fn code_block(
    id: impl Into<ElementId>,
    text: String,
    _language: Option<&str>,
    style: &TextStyle,
    font_family: &SharedString,
    bg: Option<gpui::Hsla>,
    border: Option<gpui::Hsla>,
    run_cx: RunContext,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let styled = selection_styled_text(text, Vec::new(), &run_cx);

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
        .child(SelectableText::new(
            id,
            styled,
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
