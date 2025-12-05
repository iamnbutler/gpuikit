use gpui::{
    prelude::FluentBuilder, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Svg, Window,
};
use gpuikit_theme::{ActiveTheme, Themeable};

pub fn icon_button(id: impl Into<ElementId>, icon: Svg) -> IconButton {
    IconButton::new(id, icon)
}

#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Svg,
    selected: bool,
    disabled: bool,
    handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: Svg) -> Self {
        IconButton {
            id: id.into(),
            icon,
            selected: false,
            disabled: false,
            handler: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let icon_color = match (self.selected, self.disabled) {
            (true, true) => theme.accent().opacity(0.5),
            (true, false) => theme.accent(),
            (false, true) => theme.fg_disabled(),
            (false, false) => theme.fg_muted(),
        };

        gpui::div()
            .id(self.id)
            .size(px(24.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .when(!self.disabled, |div| {
                div.hover(|div| div.bg(theme.surface_secondary()))
                    .cursor_pointer()
            })
            .when(self.disabled, |div| div.cursor_not_allowed())
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
            .child(self.icon.size(px(16.)).text_color(icon_color))
    }
}
