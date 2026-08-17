//! Field component for wrapping form inputs with labels, descriptions, and error states.

use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, IntoElement, ParentElement, Rems,
    RenderOnce, SharedString, Styled, Window,
};

/// Width of the label column in the beside layout.
const LABEL_COLUMN_WIDTH: Rems = Rems(8.0);

/// Gap between the label column and the input beside it.
const LABEL_COLUMN_GAP: Rems = Rems(0.75);

/// Creates a new Field builder.
pub fn field() -> Field {
    Field::new()
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
/// field()
///     .label("Username")
///     .description("Enter your username")
///     .required(true)
///     .child(input("username"))
/// ```
#[derive(IntoElement)]
pub struct Field {
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
    /// Create a new Field.
    pub fn new() -> Self {
        Field {
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

impl Default for Field {
    fn default() -> Self {
        Self::new()
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

    /// Dims this field's own label, description and error, and **nothing
    /// else**: a field's child is an opaque `AnyElement`, so it cannot reach
    /// the control inside it. Disable the control too —
    /// `field().disabled(true).child(textarea(…).disabled(true))`.
    ///
    /// Checked deliberately while fixing the disabled `Textarea`, which looked
    /// inert under an opacity and still took keystrokes: a `Field` puts no
    /// opacity over its child, so it is not another instance of that bug.
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

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let has_error = self.error.is_some();
        let disabled = self.disabled;

        let label_color = if disabled {
            theme.fg_disabled()
        } else if has_error {
            theme.danger()
        } else {
            theme.fg()
        };

        let label_element = self.label.map(|label_text| {
            div()
                .flex()
                .gap(rems(0.25))
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

        match self.label_position {
            LabelPosition::Above => {
                // Vertical layout: label above input
                div()
                    .flex()
                    .flex_col()
                    .gap(rems(0.375))
                    .when(disabled, |el| el.cursor_not_allowed())
                    .when_some(label_element, |container, label| container.child(label))
                    .when_some(description_element, |container, desc| container.child(desc))
                    .when_some(self.child, |container, child| container.child(child))
                    .when_some(error_element, |container, err| container.child(err))
            }
            LabelPosition::Beside => {
                // Horizontal layout: label beside input
                div()
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
                            .when_some(self.child, |container, child| {
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
