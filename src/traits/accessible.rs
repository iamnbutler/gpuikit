//! What an element announces, as a property of the element rather than of the
//! tree it renders into.

use crate::a11y::A11y;

/// An element that reports itself to assistive technology.
///
/// One method, returning the whole announcement as a value: the role, the
/// accessible name, and whatever state goes with the role. The element applies
/// it to the root element it was already building with
/// [`Announce::announce`](crate::a11y::Announce::announce) — usually as the
/// first thing after `.id(…)`, since gpui only offers a role to an element
/// that has an id.
///
/// ```ignore
/// impl Accessible for Button {
///     fn a11y(&self) -> A11y {
///         A11y::new(Role::Button).name(self.label.clone()).focusable()
///     }
/// }
///
/// impl RenderOnce for Button {
///     fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
///         // Before `self`'s fields are moved into the element.
///         let a11y = self.a11y();
///         h_stack().id(self.id).announce(a11y)/* … */
///     }
/// }
/// ```
///
/// The trait is what a test — and, later, a parent that needs to know what its
/// child is called — reads. The mechanism it feeds lives in
/// [`crate::a11y`], whose module docs are the decision record for the whole
/// convention.
pub trait Accessible {
    /// What this element announces.
    fn a11y(&self) -> A11y;
}
