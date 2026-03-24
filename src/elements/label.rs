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
//!     .child(input("password"))
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, rems, App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};

/// Creates a label element.
pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

/// Text size variants for labels.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LabelSize {
    /// Small label size (0.875rem)
    Small,
    /// Default/medium label size (1rem)
    #[default]
    Medium,
    /// Large label size (1.125rem)
    Large,
}

/// A label element for form fields and text labeling.
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    for_id: Option<ElementId>,
    required: bool,
    size: LabelSize,
    muted: bool,
}

impl Label {
    /// Creates a new label with the given text.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            for_id: None,
            required: false,
            size: LabelSize::default(),
            muted: false,
        }
    }

    /// Associates this label with a specific element ID.
    ///
    /// This is useful for accessibility and connecting labels to form inputs.
    pub fn for_id(mut self, id: impl Into<ElementId>) -> Self {
        self.for_id = Some(id.into());
        self
    }

    /// Marks this label as required, showing an asterisk indicator.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the text size of the label.
    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    /// Uses small text size.
    pub fn small(mut self) -> Self {
        self.size = LabelSize::Small;
        self
    }

    /// Uses large text size.
    pub fn large(mut self) -> Self {
        self.size = LabelSize::Large;
        self
    }

    /// Uses muted/secondary text color.
    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    fn font_size(&self) -> f32 {
        match self.size {
            LabelSize::Small => 0.875,
            LabelSize::Medium => 1.0,
            LabelSize::Large => 1.125,
        }
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = self.font_size();

        let text_color = if self.muted {
            theme.fg_muted()
        } else {
            theme.fg()
        };

        div()
            .flex()
            .items_center()
            .gap(rems(0.25))
            .text_size(rems(size))
            .line_height(rems(size * 1.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color)
            .child(self.text)
            .when(self.required, |this| {
                this.child(
                    div()
                        .text_color(theme.danger())
                        .child("*"),
                )
            })
    }
}
