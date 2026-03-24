//! Trait for elements that can have a text label.
//!
//! This trait provides a consistent API for components that display a label,
//! such as checkboxes, toggles, sliders, and form fields.

use gpui::SharedString;

/// Trait for elements that can have a text label.
///
/// Components implementing this trait can display an optional label
/// alongside their main content.
pub trait Labelable: Sized {
    /// Set the label for this element.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use gpuikit::traits::labelable::Labelable;
    ///
    /// checkbox("my-checkbox", false)
    ///     .label("Accept terms and conditions")
    /// ```
    fn label(self, label: impl Into<SharedString>) -> Self;
}
