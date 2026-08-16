//! Heading element for markdown.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, prelude::*, rems, App, ElementId, ParentElement, Styled};

use super::super::inline_style::{InlinePalette, RichText};
use super::super::selectable_text::RunRole;
use super::super::style::TextStyle;
use super::paragraph::{rich_text_run, RunContext};

/// Heading level (h1-h6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    /// The level as a number, 1 through 6 — what assistive technology is told
    /// when it asks how deep this heading sits.
    pub fn level(self) -> u8 {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(level: pulldown_cmark::HeadingLevel) -> Self {
        match level {
            pulldown_cmark::HeadingLevel::H1 => HeadingLevel::H1,
            pulldown_cmark::HeadingLevel::H2 => HeadingLevel::H2,
            pulldown_cmark::HeadingLevel::H3 => HeadingLevel::H3,
            pulldown_cmark::HeadingLevel::H4 => HeadingLevel::H4,
            pulldown_cmark::HeadingLevel::H5 => HeadingLevel::H5,
            pulldown_cmark::HeadingLevel::H6 => HeadingLevel::H6,
        }
    }
}

/// Render a heading element with plain text.
pub fn heading(text: impl Into<String>, style: &TextStyle, cx: &App) -> impl IntoElement {
    let text: String = text.into();
    let theme = cx.theme();
    let text_color = style.color.unwrap_or(theme.fg());

    div()
        .w_full()
        .text_size(rems(style.size))
        .line_height(rems(style.size * style.line_height))
        .font_weight(style.weight)
        .text_color(text_color)
        .mt(rems(style.margin_top))
        .child(text)
}

/// Render a heading element with rich text (bold, italic, strikethrough,
/// inline code, clickable links, and selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rich_heading(
    id: impl Into<ElementId>,
    rich_text: &RichText,
    level: HeadingLevel,
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
        .font_weight(style.weight)
        .text_color(text_color)
        .mt(rems(style.margin_top))
        .child(rich_text_run(
            id,
            rich_text,
            palette,
            RunRole::Heading(level.level()),
            run_cx,
        ))
}
