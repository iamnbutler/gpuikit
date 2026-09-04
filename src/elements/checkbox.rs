//! Checkbox component for gpuikit

use crate::a11y::FocusNavigation;
use crate::elements::form;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::traits::selectable::Selectable;
use gpui::{
    App, Context, ElementId, EventEmitter, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
    div, prelude::*, px,
};

/// Event emitted when the checkbox state changes
pub struct CheckboxChanged {
    pub checked: bool,
}

/// The three states the box itself can be drawn in.
///
/// `Checkbox` carries `checked` and `indeterminate` as two booleans for
/// backwards compatibility; this is the same information as one value, and it
/// is what a caller that draws boxes it does not own — a table's selection
/// column — passes to [`checkbox_box`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    /// Nothing this box stands for is checked.
    #[default]
    Unchecked,
    /// Everything this box stands for is checked.
    Checked,
    /// Some but not all of it is. Drawn as a bar rather than a tick.
    Indeterminate,
}

impl CheckState {
    /// The state a box standing for `total` things of which `selected` are
    /// checked should be drawn in.
    ///
    /// Zero of zero is `Unchecked`: a box over nothing is not "all of it".
    pub fn from_count(selected: usize, total: usize) -> Self {
        if selected == 0 || total == 0 {
            CheckState::Unchecked
        } else if selected >= total {
            CheckState::Checked
        } else {
            CheckState::Indeterminate
        }
    }

    /// What a click on a box in this state asks for.
    ///
    /// Indeterminate becomes `Checked` rather than `Unchecked` — the
    /// convention every platform toolkit follows, on the reasoning that a
    /// partial selection was arrived at by adding rather than by removing.
    pub fn toggled(self) -> Self {
        match self {
            CheckState::Checked => CheckState::Unchecked,
            CheckState::Unchecked | CheckState::Indeterminate => CheckState::Checked,
        }
    }

    /// Whether this state reads as "on" to a caller storing a bool.
    pub fn is_checked(self) -> bool {
        matches!(self, CheckState::Checked)
    }

    /// Whether this state is the partial one.
    pub fn is_indeterminate(self) -> bool {
        matches!(self, CheckState::Indeterminate)
    }
}

/// The box a checkbox draws, without the row, the label or the click handling.
///
/// `Checkbox` is an entity, so an element that draws one box per row — a
/// table's selection column — cannot mint one per frame. Without this it would
/// draw its own approximation of the box instead, which is exactly the drift
/// [`ControlMetrics::track`](crate::theme::ControlMetrics::track) exists to
/// prevent for `Switch` and `Toggle`. `Checkbox::render` goes through this, so
/// there is one box in the crate.
#[derive(IntoElement)]
pub struct CheckboxBox {
    state: CheckState,
    disabled: bool,
    size: ControlSize,
}

impl CheckboxBox {
    /// A box in the given state, on the default rung.
    pub fn new(state: CheckState) -> Self {
        Self {
            state,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    /// Draws the box in its disabled colours. Does not affect interaction —
    /// the box has none; that belongs to whatever contains it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Convenience function to create a bare checkbox box.
pub fn checkbox_box(state: CheckState) -> CheckboxBox {
    CheckboxBox::new(state)
}

impl ControlSized for CheckboxBox {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CheckboxBox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let disabled = self.disabled;
        let checked = self.state.is_checked();
        let indeterminate = self.state.is_indeterminate();

        // The row is the rung; the box is the ink inside it. Sizing the row
        // off the box is what put the checkbox on a rung of its own.
        let box_size = metrics.ink;

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

        div()
            .size(box_size)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .bg(box_bg)
            .border_1()
            .border_color(box_border)
            .rounded(metrics.radius)
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
                // The glyph sizes off the box, not off a constant, so it stays
                // proportional on every rung.
                this.child(
                    div()
                        .text_size(box_size * 0.75)
                        .line_height(box_size)
                        .text_color(check_color)
                        .child("✓"),
                )
            })
            .when(indeterminate, |this| {
                this.child(
                    div()
                        .w(box_size * 0.5)
                        .h(px(2.))
                        .bg(check_color)
                        .rounded(px(1.)),
                )
            })
    }
}

/// A checkbox component for toggling boolean values
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    indeterminate: bool,
    size: ControlSize,
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
        let metrics = theme.control(self.size);
        // The first control in the crate to adopt `crate::elements::form`'s
        // ambient context, and the whole of what adopting it costs: a
        // `Fieldset` or a `Field` around this checkbox disables it without
        // anything being threaded here by hand.
        let disabled = form::disabled_here(self.disabled);
        // The handle an enclosing `Field`'s label click lands on. Tracking it
        // is what turns that click into focus on this control rather than on a
        // handle nothing watches.
        let field_focus = form::focus_handle_here();
        let label = self.label.clone();

        let state = if self.indeterminate {
            CheckState::Indeterminate
        } else if self.checked {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };

        h_stack()
            .id(self.id.clone())
            // `debug_selector` compiles to a no-op that never calls its
            // closure unless gpui's `test-support` is on, so a consumer pays
            // nothing for it — the same trade `src/elements/table.rs` makes.
            // It is what makes "a fieldset disabled this checkbox" assertable:
            // gpui has no `aria_disabled`, so the only observable difference is
            // whether a click on the row does anything.
            .debug_selector({
                let id = self.id.clone();
                move || format!("gpuikit-checkbox-{id:?}")
            })
            .when_some(field_focus, |this, handle| {
                // `track_focus` does not make the handle a tab stop by itself
                // — the same thing `a11y::Announce` has to do for a
                // caller-supplied handle.
                this.track_focus(&handle.tab_stop(true))
                    .moves_focus_on_tab()
            })
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
                checkbox_box(state)
                    .disabled(disabled)
                    .control_size(self.size),
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

/// Convenience function to create a checkbox
pub fn checkbox(id: impl Into<ElementId>, checked: bool) -> Checkbox {
    Checkbox::new(id, checked)
}

impl Disableable for Checkbox {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Checkbox {
    fn is_selected(&self) -> bool {
        self.checked
    }

    fn selected(mut self, selected: bool) -> Self {
        self.checked = selected;
        self
    }
}

impl ControlSized for Checkbox {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Labelable for Checkbox {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}
