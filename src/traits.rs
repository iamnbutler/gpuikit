//! Shared traits: the small vocabulary elements agree on — a control size, a
//! click handler, a disabled flag — rather than a place for a trait per idea.
//!
//! There is deliberately **no `portal` module**. Positioning an overlay is
//! `gpui::anchored()`'s job; the crate's convention is `deferred()` over
//! `anchored()`, with `.offset()` rather than a margin on the child. A
//! 486-line `Portal` trait lived here for a year with zero callers.

pub mod accessible;
pub mod button;
pub mod clickable;
pub mod control_sized;
pub mod disableable;
pub mod labelable;
pub mod orientable;
pub mod selectable;
