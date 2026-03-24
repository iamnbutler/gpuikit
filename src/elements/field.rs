//! Field component for wrapping form inputs with labels, descriptions, and error states.

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::labelable::Labelable;
use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};

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

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let has_error = self.error.is_some();

        let label_element = self.label.map(|label_text| {
            div()
                .flex()
                .gap(rems(0.25))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(if has_error {
                            theme.danger()
                        } else {
                            theme.fg()
                        })
                        .child(label_text),
                )
                .when(self.required, |this| {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(theme.danger())
                            .child("*"),
                    )
                })
        });

        let description_element = self.description.map(|desc| {
            div()
                .text_xs()
                .text_color(theme.fg_muted())
                .child(desc)
        });

        let error_element = self.error.map(|err| {
            div()
                .text_xs()
                .text_color(theme.danger())
                .child(err)
        });

        match self.label_position {
            LabelPosition::Above => {
                // Vertical layout: label above input
                div()
                    .flex()
                    .flex_col()
                    .gap(rems(0.375))
                    .when_some(label_element, |container, label| {
                        container.child(label)
                    })
                    .when_some(description_element, |container, desc| {
                        container.child(desc)
                    })
                    .when_some(self.child, |container, child| {
                        container.child(child)
                    })
                    .when_some(error_element, |container, err| {
                        container.child(err)
                    })
            }
            LabelPosition::Beside => {
                // Horizontal layout: label beside input
                div()
                    .flex()
                    .flex_col()
                    .gap(rems(0.375))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(rems(0.75))
                            .when_some(label_element, |container, label| {
                                container.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(rems(0.25))
                                        .pt(rems(0.5)) // Align with input
                                        .min_w(rems(8.0))
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
                        container.child(
                            div()
                                .pl(rems(8.75)) // Align with input (label width + gap)
                                .child(err),
                        )
                    })
            }
        }
    }
}
