//! Horizontal rule/divider element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{App, Styled, div, prelude::*, px, rems};

/// Render a horizontal rule (divider) element.
pub fn divider(color: Option<gpui::Hsla>, cx: &App) -> impl IntoElement + use<> {
    div()
        .h(px(1.0))
        .my(rems(1.5))
        .bg(color.unwrap_or(cx.theme().border()))
}
