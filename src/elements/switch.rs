//! Switch component for gpuikit
//!
//! A sliding switch control for toggling boolean values, similar to iOS-style switches.

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::traits::selectable::Selectable;
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    div, prelude::*, App, Context, ElementId, EventEmitter, InteractiveElement, IntoElement,
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
    size: ControlSize,
}

impl EventEmitter<SwitchChanged> for Switch {}

impl Switch {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            label: None,
            on,
            disabled: false,
            size: ControlSize::default(),
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
        let metrics = theme.control(self.size);
        let on = self.on;
        let disabled = self.disabled;
        let label = self.label.clone();

        // Shared with `Toggle`. The two used to name their own track shapes and
        // had drifted to different ones; what still tells them apart is the
        // surface-coloured thumb and the heavier shadow below.
        let track = metrics.track();

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
            .h(metrics.height)
            // A label outside the control's own box wants more room than the
            // gap between an icon and a label inside one.
            .gap(metrics.gap * 2.0)
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
                    .w(track.width)
                    .h(track.height)
                    .bg(track_bg)
                    .border_1()
                    .border_color(track_border)
                    .rounded(track.height / 2.)
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
                            .top(track.thumb_margin)
                            .when(on, |this| this.right(track.thumb_margin))
                            .when(!on, |this| this.left(track.thumb_margin))
                            .size(track.thumb)
                            .bg(thumb_bg)
                            .rounded_full()
                            .shadow_md(),
                    ),
            )
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_size(metrics.text_size)
                        .line_height(metrics.line_height)
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

impl ControlSized for Switch {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Labelable for Switch {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}
