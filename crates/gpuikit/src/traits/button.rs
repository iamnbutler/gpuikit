use super::clickable::Clickable;

/// A button is a clickable element that dispatches some
/// handler when clicked.
pub trait Button: Clickable {
    type Variant;

    fn variant(&self) -> Self::Variant;
}
