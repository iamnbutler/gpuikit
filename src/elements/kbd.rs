use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

/// Creates a keyboard key display element.
pub fn kbd(key: impl Into<SharedString>) -> Kbd {
    Kbd::new(key)
}

/// Creates a keyboard key combination display (e.g., "Ctrl+C").
pub fn kbd_combo(keys: &[impl AsRef<str>]) -> Kbd {
    let combined = keys
        .iter()
        .map(|k| k.as_ref())
        .collect::<Vec<_>>()
        .join("+");
    Kbd::new(combined)
}

#[derive(IntoElement)]
pub struct Kbd {
    key: SharedString,
    size: ControlSize,
}

impl Kbd {
    pub fn new(key: impl Into<SharedString>) -> Self {
        Kbd {
            key: key.into(),
            size: ControlSize::default(),
        }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        div()
            // A key cap is square-ish, so its minimum width is its height —
            // and both come from the rung rather than from padding plus a line
            // box, which is what used to decide them.
            .h(metrics.height)
            .min_w(metrics.height)
            .px(metrics.padding_x)
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .font_weight(FontWeight::MEDIUM)
            .font_family("Monaco, Consolas, monospace")
            .line_height(metrics.line_height)
            .bg(theme.surface())
            .text_color(theme.fg_muted())
            .border_1()
            .border_color(theme.border())
            // The key-cap look is the shadow alone now. The heavier bottom
            // border this used to draw does not fit a declared height: gpui
            // lays out border-box, so on the Small rung a 14px line box plus
            // 1px + 2px of border overflows its 16px box.
            .shadow_sm()
            .whitespace_nowrap()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .child(self.key)
    }
}

impl ControlSized for Kbd {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}
