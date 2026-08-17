//! Shared traits: the small vocabulary elements agree on — a control size, a
//! click handler, a disabled flag — rather than a place for a trait per idea.
//!
//! There is deliberately **no `portal` module**. Positioning an overlay is
//! `gpui::anchored()`'s job, and the crate's convention for using it —
//! `deferred()` over `anchored()`, `.offset()` rather than a margin on the
//! child, the fit modes, the draw-priority ladder — is written down in
//! `docs/overlays.md` and checked by `overlay_coverage` in `src/elements.rs`.
//! A 486-line `Portal` trait lived here for a year with zero callers; see that
//! document for why it could not have had any.

pub mod button;
pub mod clickable;
pub mod control_sized;
pub mod disableable;
pub mod labelable;
pub mod orientable;
pub mod selectable;
pub mod visual_focus;
