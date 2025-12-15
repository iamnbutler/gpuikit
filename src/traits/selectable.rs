/// Trait for elements that can have a selected/toggled state.
///
/// This is commonly used for toggle buttons, checkboxes, switches,
/// and other elements that represent an on/off or active/inactive state.
pub trait Selectable {
    /// Returns whether this element is currently selected/toggled on.
    fn is_selected(&self) -> bool;

    /// Sets the selected state of this element.
    ///
    /// Returns `Self` to allow method chaining in builder patterns.
    fn selected(self, selected: bool) -> Self;
}
