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
    /// ```
    /// # use gpui::{Context, IntoElement, Render, Window, prelude::*};
    /// use gpuikit::elements::checkbox::checkbox;
    /// use gpuikit::traits::labelable::Labelable;
    /// # struct D;
    /// # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    /// cx.new(|_cx| {
    ///     checkbox("my-checkbox", false)
    ///         .label("Accept terms and conditions")
    /// })
    /// # }}
    /// # let mut tcx = gpui::TestAppContext::single();
    /// # tcx.update(gpuikit::init);
    /// # let _ = tcx.add_window_view(|_, _| D);
    /// ```
    fn label(self, label: impl Into<SharedString>) -> Self;
}
