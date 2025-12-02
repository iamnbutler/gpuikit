//! Paragraph element for markdown.

use gpui::{div, prelude::*, rems, App, ParentElement, Styled};
use gpuikit_theme::ActiveTheme;

use crate::style::TextStyle;

/// Render a paragraph element.
pub fn paragraph(text: String, style: &TextStyle, cx: &App) -> impl IntoElement {
    div()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .text_color(style.color.unwrap_or(cx.theme().fg))
        .child(text)
}
