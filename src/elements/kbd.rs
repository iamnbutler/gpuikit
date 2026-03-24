use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, px, rems, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum KbdSize {
    Small,
    #[default]
    Default,
    Large,
}

#[derive(IntoElement)]
pub struct Kbd {
    key: SharedString,
    size: KbdSize,
}

impl Kbd {
    pub fn new(key: impl Into<SharedString>) -> Self {
        Kbd {
            key: key.into(),
            size: KbdSize::Default,
        }
    }

    pub fn size(mut self, size: KbdSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = KbdSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = KbdSize::Large;
        self
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (text_size, px_val, py_val, min_width) = match self.size {
            KbdSize::Small => (rems(0.625), rems(0.25), px(1.0), rems(1.0)),
            KbdSize::Default => (rems(0.75), rems(0.375), px(2.0), rems(1.25)),
            KbdSize::Large => (rems(0.875), rems(0.5), px(3.0), rems(1.5)),
        };

        div()
            .px(px_val)
            .py(py_val)
            .min_w(min_width)
            .rounded(rems(0.25))
            .text_size(text_size)
            .font_weight(FontWeight::MEDIUM)
            .font_family("Monaco, Consolas, monospace")
            .line_height(rems(1.0))
            .bg(theme.surface())
            .text_color(theme.fg_muted())
            .border_1()
            .border_color(theme.border())
            .border_b_2()
            .shadow_sm()
            .whitespace_nowrap()
            .flex()
            .items_center()
            .justify_center()
            .child(self.key)
    }
}
