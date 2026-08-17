//! The trait a control implements to sit on a rung of the shared size scale.

use crate::theme::ControlSize;

/// A control that can be put on a rung of the shared size scale.
///
/// Implemented by every element that can share a row with another control:
/// `Button`, `IconButton`, `Checkbox`, `Switch`, `Toggle`, `Select`,
/// `TextField`, `Badge`, `Kbd`, `Input`, `Textarea` and `Field`. The rung
/// resolves through the theme
/// ([`Themeable::control`](crate::theme::Themeable::control)), so a control
/// never names a height of its own.
///
/// ```ignore
/// h_stack()
///     .child(button("save", "Save").large())
///     .child(badge("2").large())
/// ```
pub trait ControlSized: Sized {
    /// Put this control on the given rung.
    fn control_size(self, size: ControlSize) -> Self;

    /// 16px tall at a 16px root.
    fn small(self) -> Self {
        self.control_size(ControlSize::Small)
    }

    /// 20px tall at a 16px root — the default rung.
    fn medium(self) -> Self {
        self.control_size(ControlSize::Medium)
    }

    /// 24px tall at a 16px root.
    fn large(self) -> Self {
        self.control_size(ControlSize::Large)
    }
}
