//! Block quote element for markdown.

use gpui::{div, prelude::*, px, rems, App, ParentElement, Styled};
use gpuikit_theme::ActiveTheme;

use crate::style::TextStyle;

/// Render a block quote element.
pub fn block_quote(
    text: String,
    style: &TextStyle,
    border_color: Option<gpui::Hsla>,
    text_color: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .pl(rems(1.0))
        .border_l(px(3.0))
        .border_color(border_color.unwrap_or(theme.border))
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(text_color.unwrap_or(theme.fg_muted))
        .italic()
        .child(text)
}
