use super::disableable::Disableable;
use gpui::{App, ClickEvent, Window};

/// Trait for elements that can be clicked.
///
/// This trait extends `Disableable` since clickable elements typically
/// need to support being disabled to prevent interaction.
pub trait Clickable: Disableable {
    /// Set the click handler.
    ///
    /// The handler will be called when the element is clicked,
    /// unless the element is disabled.
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;
}
