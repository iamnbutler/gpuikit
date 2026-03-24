//! Label component for form fields and UI elements.
//!
//! Provides a styled label that can be used with form inputs and other UI elements.
//!
//! # Examples
//!
//! ```ignore
//! use gpuikit::elements::label::*;
//!
//! // Basic label
//! label("Email address")
//!
//! // Required field label
//! label("Username").required(true)
//!
//! // Different sizes
//! label("Small").small()
//! label("Large").large()
//!
//! // Muted variant
//! label("Optional").muted()
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, rems, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
};

/// Creates a label element.
pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

/// Size variants for the Label component.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LabelSize {
    /// Small size label (0.75rem).
    Small,
    /// Default size label (0.875rem).
    #[default]
    Default,
    /// Large size label (1rem).
    Large,
}

/// Color variants for the Label component.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LabelVariant {
    /// Default foreground color.
    #[default]
    Default,
    /// Muted/subdued color.
    Muted,
    /// Accent/highlight color.
    Accent,
    /// Danger/error color.
    Destructive,
}

/// A label element for form fields and other UI elements.
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    size: LabelSize,
    variant: LabelVariant,
    required: bool,
}

impl Label {
    /// Creates a new label with the given text.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Label {
            text: text.into(),
            size: LabelSize::Default,
            variant: LabelVariant::Default,
            required: false,
        }
    }

    /// Sets the size of the label.
    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    /// Uses small size.
    pub fn small(mut self) -> Self {
        self.size = LabelSize::Small;
        self
    }

    /// Uses large size.
    pub fn large(mut self) -> Self {
        self.size = LabelSize::Large;
        self
    }

    /// Sets the color variant.
    pub fn variant(mut self, variant: LabelVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses muted color.
    pub fn muted(mut self) -> Self {
        self.variant = LabelVariant::Muted;
        self
    }

    /// Uses accent color.
    pub fn accent(mut self) -> Self {
        self.variant = LabelVariant::Accent;
        self
    }

    /// Uses destructive/danger color.
    pub fn destructive(mut self) -> Self {
        self.variant = LabelVariant::Destructive;
        self
    }

    /// Marks the label as required (shows an asterisk).
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let text_size = match self.size {
            LabelSize::Small => rems(0.75),
            LabelSize::Default => rems(0.875),
            LabelSize::Large => rems(1.0),
        };

        let text_color = match self.variant {
            LabelVariant::Default => theme.fg(),
            LabelVariant::Muted => theme.fg_muted(),
            LabelVariant::Accent => theme.accent(),
            LabelVariant::Destructive => theme.danger(),
        };

        let mut container = div()
            .flex()
            .items_center()
            .gap(rems(0.25))
            .text_size(text_size)
            .line_height(rems(1.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color)
            .child(self.text);

        if self.required {
            container = container.child(
                div()
                    .text_color(theme.danger())
                    .child("*"),
            );
        }

        container
    }
}
