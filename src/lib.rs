//! gpuikit
//!
//! A comprehensive UI component library for GPUI applications.
//!
//! # Quick Start
//!
//! ```ignore
//! use gpui::Application;
//! use gpuikit::init;
//!
//! fn main() {
//!     Application::new()
//!         .with_assets(gpuikit::assets())
//!         .run(|cx| {
//!             init(cx);
//!             // ... your app code
//!         });
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `editor` - Enables the editor component with syntax highlighting support

use gpui::App;
use rust_embed::RustEmbed;

// Core modules
pub mod elements;
pub mod error;
pub mod fs;
pub mod icons;
pub mod keymap;
pub mod layout;
pub mod markdown;
pub mod resource;
pub mod theme;
pub mod traits;
pub mod utils;

// Feature-gated editor module
#[cfg(feature = "editor")]
pub mod editor;

pub use icons::Icons as DefaultIcons;

/// Embedded assets for gpuikit (icons, fonts, etc.)
#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

/// Returns the gpuikit asset source for use with `Application::new().with_assets()`.
///
/// # Example
/// ```ignore
/// Application::new()
///     .with_assets(gpuikit::assets())
///     .run(|cx| {
///         gpuikit::init(cx);
///         // ...
///     });
/// ```
pub fn assets() -> resource::ResourceSource<Assets> {
    resource::ResourceSource::new()
}

/// Initialize gpuikit - sets up themes and global state.
///
/// This must be called as soon as possible after your `gpui::Application` is created.
/// Make sure to also call `.with_assets(gpuikit::Assets)` on your Application.
///
/// # Panics
/// Calling a gpuikit component before initialization will panic.
pub fn init(cx: &mut App) {
    theme::init(cx);
    utils::element_manager::init(cx);
}
