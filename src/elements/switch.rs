//! Switch component for gpuikit
//!
//! A sliding switch control for toggling boolean values, similar to iOS-style switches.

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::traits::selectable::Selectable;
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    div, prelude::*, rems, App, Context, ElementId, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

/// Event emitted when the switch state changes
pub struct SwitchChanged {
    pub on: bool,
}

/// A sliding switch component for toggling between on/off states
///
/// Similar to a toggle, but with a more pronounced sliding switch appearance.
///
/// # Example
///
/// ```ignore
/// use gpuikit::elements::switch::switch;
///
/// switch("dark-mode", true).label("Dark Mode")
/// ```
pub struct Switch {
    id: ElementId,
    label: Option<SharedString>,
    on: bool,
    disabled: bool,
}

impl EventEmitter<SwitchChanged> for Switch {}

impl Switch {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            label: None,
            on,
            disabled: false,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, on: bool, cx: &mut Context<Self>) {
        if self.on != on {
            self.on = on;
            cx.emit(SwitchChanged { on: self.on });
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.set_on(!self.on, cx);
    }

    fn on_click(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.toggle(cx);
        }
    }
}

impl Render for Switch {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let on = self.on;
        let disabled = self.disabled;
        let label = self.label.clone();

        // Switch dimensions - wider and shorter than toggle for a more pronounced switch look
        let track_width = rems(2.75);
        let track_height = rems(1.5);
        let thumb_size = rems(1.25);
        let thumb_margin = rems(0.125);

        let track_bg = if disabled {
            theme.surface_tertiary()
        } else if on {
            theme.accent()
        } else {
            theme.surface_secondary()
        };

        let thumb_bg = if disabled {
            theme.fg_disabled()
        } else {
            theme.surface()
        };

        let track_border = if disabled {
            theme.border_subtle()
        } else if on {
            theme.accent()
        } else {
            theme.border()
        };

        h_stack()
            .id(self.id.clone())
            .gap(rems(0.5))
            .items_center()
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.on_click(window, cx);
                    }))
            })
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .child(
                div()
                    .relative()
                    .w(track_width)
                    .h(track_height)
                    .bg(track_bg)
                    .border_1()
                    .border_color(track_border)
                    .rounded(track_height / 2.)
                    .when(!disabled, |this| {
                        this.hover(|style| {
                            style.border_color(if on {
                                theme.accent()
                            } else {
                                theme.border_secondary()
                            })
                        })
                    })
                    .child(
                        div()
                            .absolute()
                            .top(thumb_margin)
                            .when(on, |this| this.right(thumb_margin))
                            .when(!on, |this| this.left(thumb_margin))
                            .size(thumb_size)
                            .bg(thumb_bg)
                            .rounded_full()
                            .shadow_md(),
                    ),
            )
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(if disabled {
                            theme.fg_disabled()
                        } else {
                            theme.fg()
                        })
                        .child(label),
                )
            })
    }
}

/// Convenience function to create a switch
pub fn switch(id: impl Into<ElementId>, on: bool) -> Switch {
    Switch::new(id, on)
}

/// Convenience function to create a switch with auto-generated ID
pub fn switch_auto(cx: &App, on: bool) -> Switch {
    Switch::new(cx.next_id_named("switch"), on)
}

impl Disableable for Switch {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Switch {
    fn is_selected(&self) -> bool {
        self.on
    }

    fn selected(mut self, selected: bool) -> Self {
        self.on = selected;
        self
    }
}

impl Labelable for Switch {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}
