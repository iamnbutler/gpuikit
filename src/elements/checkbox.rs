//! Checkbox component for gpuikit

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, px, rems, Context, ElementId, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

/// Event emitted when the checkbox state changes
pub struct CheckboxChanged {
    pub checked: bool,
}

/// A checkbox component for toggling boolean values
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    indeterminate: bool,
}

impl EventEmitter<CheckboxChanged> for Checkbox {}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked,
            disabled: false,
            indeterminate: false,
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

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }

    pub fn set_checked(&mut self, checked: bool, cx: &mut Context<Self>) {
        if self.checked != checked {
            self.checked = checked;
            self.indeterminate = false;
            cx.emit(CheckboxChanged {
                checked: self.checked,
            });
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.set_checked(!self.checked, cx);
    }

    fn on_click(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.toggle(cx);
        }
    }
}

impl Render for Checkbox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let checked = self.checked;
        let disabled = self.disabled;
        let indeterminate = self.indeterminate;
        let label = self.label.clone();

        let box_size = rems(1.0);

        let box_bg = if disabled {
            theme.surface_tertiary()
        } else if checked || indeterminate {
            theme.accent()
        } else {
            theme.surface()
        };

        let box_border = if disabled {
            theme.border_subtle()
        } else if checked || indeterminate {
            theme.accent()
        } else {
            theme.border()
        };

        let check_color = if disabled {
            theme.fg_disabled()
        } else {
            theme.surface()
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
                    .size(box_size)
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(box_bg)
                    .border_1()
                    .border_color(box_border)
                    .rounded(px(3.))
                    .when(!disabled, |this| {
                        this.hover(|style| {
                            style.border_color(if checked || indeterminate {
                                theme.accent()
                            } else {
                                theme.border_secondary()
                            })
                        })
                    })
                    .when(checked && !indeterminate, |this| {
                        this.child(div().text_xs().text_color(check_color).child("✓"))
                    })
                    .when(indeterminate, |this| {
                        this.child(div().w(rems(0.5)).h(px(2.)).bg(check_color).rounded(px(1.)))
                    }),
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

/// Convenience function to create a checkbox
pub fn checkbox(id: impl Into<ElementId>, checked: bool) -> Checkbox {
    Checkbox::new(id, checked)
}
