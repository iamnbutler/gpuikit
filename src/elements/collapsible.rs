//! Collapsible component for expandable/collapsible content sections

use crate::icons::Icons;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    div, prelude::*, px, rems, AnyElement, App, Context, ElementId, EventEmitter, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use std::rc::Rc;

/// Event emitted when the collapsible open state changes
pub struct CollapsibleChanged {
    pub open: bool,
}

/// A render callback that produces an element
pub type RenderCallback = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// A collapsible component for expandable/collapsible content sections
pub struct Collapsible {
    id: ElementId,
    trigger_label: Option<SharedString>,
    trigger_render: Option<RenderCallback>,
    content_render: Option<RenderCallback>,
    open: bool,
    disabled: bool,
    show_indicator: bool,
    on_toggle: Option<Rc<dyn Fn(bool, &mut Window, &mut App)>>,
}

impl EventEmitter<CollapsibleChanged> for Collapsible {}

impl Collapsible {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_label: None,
            trigger_render: None,
            content_render: None,
            open: false,
            disabled: false,
            show_indicator: true,
            on_toggle: None,
        }
    }

    /// Set a custom trigger element via a render callback
    pub fn trigger(mut self, render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.trigger_render = Some(Rc::new(render));
        self
    }

    /// Set a simple text label as the trigger
    pub fn trigger_label(mut self, label: impl Into<SharedString>) -> Self {
        self.trigger_label = Some(label.into());
        self
    }

    /// Set the collapsible content via a render callback
    pub fn content(mut self, render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.content_render = Some(Rc::new(render));
        self
    }

    /// Set the initial open state (for uncontrolled mode)
    pub fn default_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set whether to show the chevron indicator
    pub fn show_indicator(mut self, show: bool) -> Self {
        self.show_indicator = show;
        self
    }

    /// Register a callback for when the open state changes
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            if let Some(on_toggle) = &self.on_toggle {
                let on_toggle = on_toggle.clone();
                on_toggle(self.open, window, cx);
            }
            cx.emit(CollapsibleChanged { open: self.open });
            cx.notify();
        }
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_open(!self.open, window, cx);
    }

    fn on_click(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.toggle(window, cx);
        }
    }
}

impl Render for Collapsible {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open;
        let disabled = self.disabled;
        let show_indicator = self.show_indicator;

        // Extract theme colors before any callbacks to avoid borrow issues
        let theme = cx.theme();
        let fg_color = theme.fg();
        let fg_muted = theme.fg_muted();
        let surface_secondary = theme.surface_secondary();

        // Build trigger element
        let trigger_element = if let Some(ref render) = self.trigger_render {
            render(window, cx)
        } else if let Some(ref label) = self.trigger_label {
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(fg_color)
                .child(label.clone())
                .into_any_element()
        } else {
            div()
                .text_sm()
                .text_color(fg_color)
                .child("Toggle")
                .into_any_element()
        };

        // Build content element if open
        let content_element = if open {
            self.content_render.as_ref().map(|render| render(window, cx))
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                h_stack()
                    .id(self.id.clone())
                    .w_full()
                    .gap(rems(0.5))
                    .items_center()
                    .py(rems(0.5))
                    .when(!disabled, |this| {
                        this.cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.prevent_default()
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.on_click(window, cx);
                            }))
                            .hover(move |style| style.bg(surface_secondary))
                    })
                    .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
                    .rounded(rems(0.25))
                    .px(rems(0.5))
                    .when(show_indicator, |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(16.))
                                .child(if open {
                                    Icons::chevron_down()
                                        .size(px(14.))
                                        .text_color(fg_muted)
                                } else {
                                    Icons::chevron_right()
                                        .size(px(14.))
                                        .text_color(fg_muted)
                                }),
                        )
                    })
                    .child(div().flex_1().child(trigger_element)),
            )
            .when_some(content_element, |this, content| {
                this.child(
                    div()
                        .pl(if show_indicator { rems(1.5) } else { rems(0.5) })
                        .pr(rems(0.5))
                        .pb(rems(0.5))
                        .child(content),
                )
            })
    }
}

/// Convenience function to create a collapsible
pub fn collapsible(id: impl Into<ElementId>) -> Collapsible {
    Collapsible::new(id)
}

impl Disableable for Collapsible {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
