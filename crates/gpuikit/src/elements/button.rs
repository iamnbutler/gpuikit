use gpui::{
    div, rems, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window,
};
use gpuikit_theme::ActiveTheme;

use crate::utils::element_manager::ElementManagerExt;

pub fn button(cx: &App, label: impl Into<SharedString>) -> Button {
    let label = label.into();
    let id = cx.next_id_named(label.clone());
    Button::new(id, label)
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        let label = label.into();

        Button {
            id,
            label,
            disabled: false,
            handler: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut button = div()
            .id(self.id)
            .h(rems(1.0))
            .px(rems(0.125))
            .gap(rems(0.125))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(rems(0.125))
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .bg(cx.theme().button_bg)
            .text_color(theme.fg)
            .whitespace_nowrap();

        if !self.disabled {
            button = button
                .hover(|div| div.bg(cx.theme().button_bg_hover))
                .active(|div| div.bg(cx.theme().button_bg_active))
        } else {
            button = button
                .opacity(0.65)
                .cursor_not_allowed()
                .text_color(theme.fg_muted)
        }

        if !self.disabled {
            if let Some(handler) = self.handler {
                button = button
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(move |event, window, cx| {
                        cx.stop_propagation();
                        handler(event, window, cx)
                    });
            }
        }

        button.child(self.label)
    }
}
