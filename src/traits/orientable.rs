//! Trait for elements that can have horizontal or vertical orientation.
//!
//! This trait provides a consistent API for components that can be laid out
//! in different orientations, such as radio groups, separators, and button groups.

/// Orientation of an element.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Horizontal layout (left to right).
    Horizontal,
    /// Vertical layout (top to bottom).
    #[default]
    Vertical,
}

/// Trait for elements that can have horizontal or vertical orientation.
///
/// Components implementing this trait can be laid out in either
/// horizontal or vertical direction.
pub trait Orientable: Sized {
    /// Set the orientation of this element.
    fn orientation(self, orientation: Orientation) -> Self;

    /// Set this element to horizontal orientation.
    fn horizontal(self) -> Self {
        self.orientation(Orientation::Horizontal)
    }

    /// Set this element to vertical orientation.
    fn vertical(self) -> Self {
        self.orientation(Orientation::Vertical)
    }
}
