//! Label component for form field labels.
//!
//! A simple text label component used to associate labels with form inputs.
//!
//! # Examples
//!
//! ```ignore
//! use gpuikit::elements::label::*;
//!
//! // Simple usage
//! label("Username")
//!
//! // With builder pattern
//! label("Email")
//!     .for_id("email-input")
//!     .required(true)
//!
//! // Usage with form field
//! v_stack()
//!     .child(label("Password").required(true))
//!     .child(input(&password_state, cx))
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, rems, App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};

/// Creates a new label element.
pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

/// A label element for form fields.
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    for_id: Option<ElementId>,
    required: bool,
}

impl Label {
    /// Creates a new label with the given text.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            for_id: None,
            required: false,
        }
    }

    /// Associates this label with a specific element ID.
    ///
    /// This is useful for accessibility, linking the label to its form control.
    pub fn for_id(mut self, id: impl Into<ElementId>) -> Self {
        self.for_id = Some(id.into());
        self
    }

    /// Marks this label as required, showing an asterisk indicator.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let base = div()
            .flex()
            .flex_row()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .line_height(rems(1.0))
            .text_color(theme.fg());

        if self.required {
            base.child(self.text).child(
                div()
                    .ml(rems(0.125))
                    .text_color(theme.danger())
                    .child("*"),
            )
        } else {
            base.child(self.text)
        }
    }
}
