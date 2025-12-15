//! Simple text tooltip component.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, px, AnyView, App, AppContext, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window,
};

/// A simple text tooltip.
pub struct Tooltip {
    text: SharedString,
}

impl Tooltip {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }

    /// Returns a closure suitable for use with `.tooltip()` on elements.
    pub fn text(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
        let text = text.into();
        move |_, cx| cx.new(|_| Tooltip::new(text.clone())).into()
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();

        div().pl(px(8.)).pt(px(10.)).child(
            div()
                .py(px(4.))
                .px(px(8.))
                .bg(theme.surface())
                .border_1()
                .border_color(theme.border())
                .rounded(px(4.))
                .shadow_md()
                .text_sm()
                .text_color(theme.fg())
                .child(self.text.clone()),
        )
    }
}

/// Convenience function for creating a text tooltip.
pub fn tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
    Tooltip::text(text)
}
