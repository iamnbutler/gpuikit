//! gpuikit

use gpui::App;
pub use gpuikit_theme as theme;

pub mod elements;
pub mod error;
pub mod fs;
pub mod layout;
pub mod resource;
pub mod traits;
pub mod utils;

/// Initialize gpuikit - this sets up & loads themes, sets up global state, etc.
///
/// This must be called as soon as possible after your `gpui::Application` is created,
/// as calling a gpuikit component before initialization will panic.
pub fn init(cx: &mut App) {
    theme::init(cx);
    utils::element_manager::init(cx);
}
