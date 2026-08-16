//! Block quote element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, px, rems, App, ElementId, ParentElement, Styled};

use super::super::inline_style::{InlinePalette, RichText};
use super::super::selectable_text::RunRole;
use super::super::style::TextStyle;
use super::paragraph::{rich_text_run, RunContext};

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

/// Render a block quote element with rich text (bold, italic, strikethrough,
/// inline code, clickable links, and selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rich_block_quote(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    style: &TextStyle,
    border_color: Option<gpui::Hsla>,
    text_color: Option<gpui::Hsla>,
    palette: &InlinePalette,
    run_cx: RunContext,
    cx: &App,
) -> impl IntoElement {
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
        .child(rich_text_run(
            id,
            rich_text,
            palette,
            RunRole::Quote,
            run_cx,
        ))
}
