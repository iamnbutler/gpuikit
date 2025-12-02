//! Link element for markdown.

use gpui::{
    div, prelude::*, App, CursorStyle, ElementId, InteractiveElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled,
};
use gpuikit_theme::ActiveTheme;

/// Render a link element.
///
/// Links are rendered with accent color and underline, and open in the default browser on click.
pub fn link(
    id: impl Into<ElementId>,
    text: String,
    url: SharedString,
    color: Option<gpui::Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let link_color = color.unwrap_or(theme.accent);

    div()
        .id(id)
        .text_color(link_color)
        .underline()
        .text_decoration_color(link_color)
        .cursor(CursorStyle::PointingHand)
        .hover(|style| style.opacity(0.8))
        .on_click({
            let url = url.clone();
            move |_event, _window, cx| {
                cx.open_url(&url);
            }
        })
        .child(text)
}
