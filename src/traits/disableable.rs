/// Trait for elements that can be disabled.
///
/// This is a common pattern for interactive elements that should
/// be able to prevent user interaction when in a disabled state.
pub trait Disableable: Sized {
    /// Check if this element is disabled.
    fn is_disabled(&self) -> bool;

    /// Set the disabled state of this element.
    fn disabled(self, disabled: bool) -> Self;
}
