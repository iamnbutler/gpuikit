//! Field component for wrapping form inputs with labels, descriptions, and error states.
//!
//! A `Field` does two things a control beside it cannot do for itself: it
//! *names* it, and it opens an ambient [`FormContext`] carrying that name, the
//! field's `disabled`, and the focus handle its label click lands on. See
//! [`crate::elements::form`] for why that is ambient rather than a prop.

use crate::a11y::{A11y, Announce};
use crate::element_id;
use crate::elements::form::{self, FormContext, WithFormContext};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, ElementId, InteractiveElement, IntoElement,
    ParentElement, Rems, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window,
};

/// Width of the label column in the beside layout.
const LABEL_COLUMN_WIDTH: Rems = Rems(8.0);

/// Gap between the label column and the input beside it.
const LABEL_COLUMN_GAP: Rems = Rems(0.75);

/// Creates a new Field builder with the given id.
pub fn field(id: impl Into<ElementId>) -> Field {
    Field::new(id)
}

/// Label position relative to the input.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    /// Label is displayed above the input (default).
    #[default]
    Above,
    /// Label is displayed beside the input (horizontal layout).
    Beside,
}

/// A field component for wrapping form inputs with labels, descriptions, and error states.
///
/// # Example
///
/// ```ignore
/// field("username")
///     .label("Username")
///     .description("Enter your username")
///     .required(true)
///     .child(input("username"))
/// ```
#[derive(IntoElement)]
pub struct Field {
    id: ElementId,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    label_position: LabelPosition,
    child: Option<AnyElement>,
    disabled: bool,
    size: ControlSize,
}

impl Field {
    /// Create a new Field with the given id.
    ///
    /// The id is required — it carries the field's accessibility node, scopes
    /// the label's own id, and keys the focus handle the label publishes. See
    /// [`crate::element_id`] for the rule it has to satisfy.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Field {
            id: id.into(),
            label: None,
            description: None,
            error: None,
            required: false,
            label_position: LabelPosition::default(),
            child: None,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    /// Set the description/help text for this field.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set an error message for this field.
    /// When set, the field will display in an error state.
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Mark this field as required.
    /// Displays a required indicator next to the label.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the label position (above or beside the input).
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }

    /// Set the child element (typically a form input).
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl Labelable for Field {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Disableable for Field {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Dims this field's own label, description and error, **and** publishes
    /// `disabled` to the control inside it through the ambient
    /// [`FormContext`].
    ///
    /// A control that reads [`form::disabled_here`] therefore needs nothing
    /// said about it twice. A control that has not adopted the context still
    /// needs its own `disabled(true)` — a field's child is an opaque
    /// `AnyElement`, so this element cannot reach into it.
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for Field {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Accessible for Field {
    /// A `Role::Group` named by the label.
    ///
    /// This is label association expressed as its *result* rather than as the
    /// relation: gpui has no `labelled_by` builder — accesskit has the
    /// relation, gpui's `AriaProperties` has no field for it — so the field
    /// announces the name over the label and the control, and republishes it
    /// through [`FormContext::name`] for the control to announce as its own.
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::Group);
        match &self.label {
            Some(label) => a11y.name(label.clone()),
            None => a11y,
        }
    }
}

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let a11y = self.a11y();
        // Before the theme, which borrows `cx` immutably for the rest of the
        // function.
        let focus_handle = form::field_focus_handle(&self.id, cx);
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let has_error = self.error.is_some();
        // Its own `disabled`, over any group around it. One line, and it is
        // the whole of what adopting the cascade costs.
        let disabled = form::disabled_here(self.disabled);

        let label_color = if disabled {
            theme.fg_disabled()
        } else if has_error {
            theme.danger()
        } else {
            theme.fg()
        };

        let label_element = self.label.clone().map(|label_text| {
            let handle = focus_handle.clone();
            div()
                .id(element_id::scoped(&self.id, "label"))
                // `debug_selector` compiles to a no-op that never calls its
                // closure unless gpui's `test-support` is on, so a consumer
                // pays nothing for it — the same trade `src/elements/table.rs`
                // makes. It is what makes "a click on the label lands focus on
                // the control" assertable at all.
                .debug_selector(|| "gpuikit-field-label".into())
                .flex()
                .gap(rems(0.25))
                // Clicking a label focuses the control it names. A control
                // that has not adopted `form::focus_handle_here` tracks no
                // such handle, and the click is inert rather than wrong.
                .cursor_pointer()
                .on_click(move |_, window, cx| window.focus(&handle, cx))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(label_color)
                        .child(label_text),
                )
                .when(self.required && !disabled, |this| {
                    this.child(div().text_sm().text_color(theme.danger()).child("*"))
                })
        });

        let description_element = self.description.map(|desc| {
            div()
                .text_xs()
                .text_color(if disabled {
                    theme.fg_disabled()
                } else {
                    theme.fg_muted()
                })
                .child(desc)
        });

        let error_element = self
            .error
            .map(|err| div().text_xs().text_color(theme.danger()).child(err));

        // What the field tells the control beside it: what it is called, that
        // it is disabled, and which handle its label clicks.
        let context = {
            let context = FormContext::new()
                .disabled(disabled)
                .focus_handle(focus_handle);
            match self.label {
                Some(label) => context.name(label),
                None => context,
            }
        };
        let child = self
            .child
            .map(|child| WithFormContext::new(context, child));

        match self.label_position {
            LabelPosition::Above => {
                // Vertical layout: label above input
                div()
                    .id(self.id)
                    .announce(a11y)
                    .flex()
                    .flex_col()
                    .gap(rems(0.375))
                    .when(disabled, |el| el.cursor_not_allowed())
                    .when_some(label_element, |container, label| container.child(label))
                    .when_some(description_element, |container, desc| container.child(desc))
                    .when_some(child, |container, child| container.child(child))
                    .when_some(error_element, |container, err| container.child(err))
            }
            LabelPosition::Beside => {
                // Horizontal layout: label beside input
                div()
                    .id(self.id)
                    .announce(a11y)
                    .flex()
                    .flex_col()
                    .gap(rems(0.375))
                    .when(disabled, |el| el.cursor_not_allowed())
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(LABEL_COLUMN_GAP)
                            .when_some(label_element, |container, label| {
                                container.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        .gap(rems(0.25))
                                        // The label's box is exactly the
                                        // input's box, so the two lines of
                                        // text centre against each other —
                                        // rather than the old
                                        // `pt(rems(0.5)) // Align with input`,
                                        // which was a guess at a height the
                                        // input did not declare.
                                        .min_h(metrics.height)
                                        .min_w(LABEL_COLUMN_WIDTH)
                                        .child(label)
                                        .when_some(description_element, |this, desc| {
                                            this.child(desc)
                                        }),
                                )
                            })
                            .when_some(child, |container, child| {
                                container.child(div().flex_1().child(child))
                            }),
                    )
                    .when_some(error_element, |container, err| {
                        container.child(div().pl(LABEL_COLUMN_WIDTH + LABEL_COLUMN_GAP).child(err))
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::test_support::announced;
    use gpui::TestAppContext;

    #[gpui::test]
    fn a_field_announces_its_label_as_a_group(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| {
            announced(field("street").label("Street"), window, cx)
        });

        assert_eq!(announced.role, Some(Role::Group));
        assert_eq!(announced.name(), Some("Street"));
        assert_eq!(announced.id, Some(ElementId::Name("street".into())));
    }

    /// The name the field publishes to the control beside it is the same
    /// string it announces itself — there is no second place for the two to
    /// disagree.
    #[test]
    fn a_field_publishes_its_label_as_the_ambient_name() {
        let field = field("street").label("Street");

        assert_eq!(
            field.a11y().accessible_name().map(|name| name.to_string()),
            Some("Street".to_string())
        );
    }

    /// A field inside a disabled group is disabled, without being told.
    #[test]
    fn a_field_inherits_an_enclosing_groups_disabled() {
        form::scope(FormContext::new().disabled(true), || {
            assert!(form::disabled_here(field("street").is_disabled()));
        });

        assert!(!form::disabled_here(field("street").is_disabled()));
    }
}
