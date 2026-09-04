//! The trait a control implements to sit on a rung of the shared size scale.

use crate::theme::ControlSize;

/// A control that can be put on a rung of the shared size scale.
///
/// Implemented by every element that can share a row with another control.
/// The list is not repeated here — it went stale twice, naming twelve
/// implementors when there were twenty-two — so `grep "impl ControlSized"` is
/// the answer, and
/// `crate::elements::control_size_tests::every_sized_control_on_a_row_is_the_same_height`
/// is the one that says which of them are held to the rung and which are not
/// yet. The rung resolves through the theme
/// ([`Themeable::control`](crate::theme::Themeable::control)), so a control
/// never names a height of its own.
///
/// ```
/// # use gpui::{Context, IntoElement, Render, Window, prelude::*};
/// use gpuikit::elements::badge::badge;
/// use gpuikit::elements::button::button;
/// use gpuikit::layout::h_stack;
/// use gpuikit::traits::control_sized::ControlSized;
/// # struct D;
/// # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
/// h_stack()
///     .child(button("save", "Save").large())
///     .child(badge("2").large())
/// # }}
/// # let mut tcx = gpui::TestAppContext::single();
/// # tcx.update(gpuikit::init);
/// # let _ = tcx.add_window_view(|_, _| D);
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
