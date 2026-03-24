//! Label component for form field labels

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, rems, App, Div, ElementId, FontWeight, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};

/// Creates a label with the given text
pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

/// A label component for form fields and text labeling
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    for_id: Option<ElementId>,
    required: bool,
    disabled: bool,
}

impl Label {
    /// Create a new label with the given text
    pub fn new(text: impl Into<SharedString>) -> Self {
        Label {
            text: text.into(),
            for_id: None,
            required: false,
            disabled: false,
        }
    }

    /// Associate this label with a specific element ID
    pub fn for_id(mut self, id: impl Into<ElementId>) -> Self {
        self.for_id = Some(id.into());
        self
    }

    /// Mark this label as required, showing an indicator (asterisk)
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the disabled state of the label
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let text_color = if self.disabled {
            theme.fg_disabled()
        } else {
            theme.fg()
        };

        div()
            .flex()
            .items_center()
            .gap(rems(0.25))
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color)
            .child(self.text)
            .when(self.required, |this: Div| {
                this.child(div().text_color(theme.danger()).child("*"))
            })
    }
}
