use crate::a11y::{A11y, Announce};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::{
    layout::h_stack, traits, traits::accessible::Accessible, traits::control_sized::ControlSized,
    traits::disableable::Disableable,
};
use gpui::{
    prelude::FluentBuilder, AnyView, App, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    let label = label.into();
    let id = id.into();
    Button::new(id, label)
}

// todo: style through ButtonVariant
#[derive(Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    size: ControlSize,
    handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    tooltip: Option<Box<dyn Fn(&mut Window, &mut App) -> AnyView + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        let label = label.into();

        Button {
            id,
            label,
            disabled: false,
            size: ControlSize::default(),
            handler: None,
            tooltip: None,
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

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Box::new(tooltip));
        self
    }
}

/// The worked example for `crate::a11y`: the label is the accessible name, so
/// there is no second string to keep in step with it, and a button built with
/// an empty label is a `debug_assert!` rather than a control that announces
/// "button" and nothing else.
///
/// Nothing here reports `disabled` — gpui has no `aria_disabled` (see the
/// `a11y` module docs). What a disabled button does report is the absence of
/// `Action::Click`, because `render` drops the click handler below.
impl Accessible for Button {
    fn a11y(&self) -> A11y {
        A11y::new(Role::Button).name(self.label.clone())
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        // Taken before `self`'s fields are moved into the element below.
        let a11y = self.a11y();

        h_stack()
            .id(self.id)
            .announce(a11y)
            .h(metrics.height)
            .px(metrics.padding_x)
            .gap(metrics.gap)
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .line_height(metrics.line_height)
            .font_weight(FontWeight::MEDIUM)
            .bg(theme.button_bg())
            .text_color(theme.fg())
            .whitespace_nowrap()
            .when(!self.disabled, |button| {
                button
                    .hover(|div| div.bg(theme.button_bg_hover()))
                    .active(|div| div.bg(theme.button_bg_active()))
                    .cursor_pointer()
            })
            .when(self.disabled, |button| {
                button
                    .opacity(0.65)
                    .cursor_not_allowed()
                    .text_color(theme.fg_muted())
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
            .when_some(self.tooltip, |button, tooltip| button.tooltip(tooltip))
            .child(self.label)
    }
}

impl traits::clickable::Clickable for Button {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
}

impl traits::button::Button for Button {
    type Variant = ButtonVariant;

    fn variant(&self) -> Self::Variant {
        ButtonVariant::default()
    }
}

impl Disableable for Button {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for Button {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::test_support::announced;
    use gpui::{accesskit, ElementId, TestAppContext};

    /// The whole announcement: a role, and the label as the name.
    #[gpui::test]
    fn a_button_announces_its_label(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| announced(button("save", "Save"), window, cx));

        assert_eq!(announced.role, Some(Role::Button));
        assert_eq!(announced.name(), Some("Save"));
    }

    /// gpui builds a node only for an element that has *both* a role and an
    /// id, and it hashes the id path into the node id. So the role has to sit
    /// on the element carrying the caller's id, not on some inner box.
    #[gpui::test]
    fn the_role_sits_on_the_element_the_caller_named(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| announced(button("save", "Save"), window, cx));

        assert_eq!(announced.id, Some(ElementId::Name("save".into())));
        assert!(announced.node.is_some(), "an id and a role make a node");
    }

    /// With no `aria_disabled` in gpui, the click action is the only thing
    /// that tells the two apart — see the `a11y` module docs, section 3.
    #[gpui::test]
    fn a_disabled_button_offers_no_click_action(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let (enabled, disabled) = cx.update(|window, cx| {
            (
                announced(button("save", "Save").on_click(|_, _, _| {}), window, cx),
                announced(
                    button("save", "Save").on_click(|_, _, _| {}).disabled(true),
                    window,
                    cx,
                ),
            )
        });

        assert!(enabled.supports(accesskit::Action::Click));
        assert!(!disabled.supports(accesskit::Action::Click));
        assert_eq!(
            disabled.name(),
            Some("Save"),
            "a disabled button is still announced, just not actionable"
        );
    }
}
