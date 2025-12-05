//! gpuikit
//!
//! A comprehensive UI component library for GPUI applications.
//!
//! # Quick Start
//!
//! ```ignore
//! use gpui::Application;
//! use gpuikit::{Assets, init};
//!
//! fn main() {
//!     Application::new()
//!         .with_assets(Assets)
//!         .run(|cx| {
//!             init(cx);
//!             // ... your app code
//!         });
//! }
//! ```

use gpui::App;
use rust_embed::RustEmbed;

pub use gpuikit_theme as theme;

pub mod elements;
pub mod error;
pub mod fs;
pub mod icons;
pub mod layout;
pub mod resource;
pub mod traits;
pub mod utils;

pub use icons::Icons as DefaultIcons;

/// Embedded assets for gpuikit (icons, fonts, etc.)
///
/// Must be passed to `Application::new().with_assets()` before calling `.run()`.
///
/// # Example
/// ```ignore
/// Application::new()
///     .with_assets(gpuikit::Assets)
///     .run(|cx| {
///         gpuikit::init(cx);
///         // ...
///     });
/// ```
#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

/// Initialize gpuikit - sets up themes and global state.
///
/// This must be called as soon as possible after your `gpui::Application` is created.
/// Make sure to also call `.with_assets(gpuikit::Assets)` on your Application.
///
/// # Panics
/// Calling a gpuikit component before initialization will panic.
pub fn init(cx: &mut App) {
    gpuikit_theme::init(cx);
    utils::element_manager::init(cx);
}
