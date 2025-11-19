use gpui::{
    div, prelude::FluentBuilder, rems, App, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use gpuikit_theme::ActiveTheme;

pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    let label = label.into();
    let id = id.into();
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

        div()
            .id(self.id)
            .h(rems(1.0))
            .px(rems(0.5))
            .gap(rems(0.25))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(rems(0.25))
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .bg(theme.button_bg)
            .text_color(theme.fg)
            .whitespace_nowrap()
            .when(!self.disabled, |button| {
                button
                    .hover(|div| div.bg(theme.button_bg_hover))
                    .active(|div| div.bg(theme.button_bg_active))
                    .cursor_pointer()
            })
            .when(self.disabled, |button| {
                button
                    .opacity(0.65)
                    .cursor_not_allowed()
                    .text_color(theme.fg_muted)
            })
            .when_some(
                self.handler.filter(|_| !self.disabled),
                |button, handler| {
                    button
                        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                        .on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx)
                        })
                },
            )
            .child(self.label)
    }
}
