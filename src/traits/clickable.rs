use gpui::{App, ClickEvent, Window};

/// Trait for elements that can be clicked
pub trait Clickable {
    /// Check if this element is disabled
    fn disabled(&self) -> bool;

    /// Set the click handler
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;
}
