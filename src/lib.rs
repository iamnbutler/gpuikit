#![allow(missing_docs)]
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
//! All features are off by default.
//!
//! - `editor` — the editor component, and the syntect-backed syntax
//!   highlighting markdown code fences use once an app calls
//!   `markdown::init_code_highlighting` (itself gated on this feature)
//! - `stitch` — closes the syntax a partially streamed markdown document leaves
//!   open (`**bold`, `[label](htt`) before parsing, so streaming text does not
//!   flicker between literal markers and styled text. Pulls in
//!   [mdstitch](https://docs.rs/mdstitch), which **requires Rust 1.95**;
//!   [`markdown::preprocessing_available`] reports which build you got
//! - `runtime_shaders` — compiles Metal shaders at runtime rather than at build
//!   time, so a macOS build needs no Xcode Metal toolchain
//! - `schema` — adds the `schemars` dependency. Nothing here derives
//!   `JsonSchema` yet, so today this only affects your dependency graph
//!
//! # Minimum Rust version
//!
//! This crate declares `rust-version = "1.85"`, which is a statement about its
//! own source — async closures, and gpui's edition 2024 — rather than a
//! guarantee about a whole build. gpuikit is edition 2021, so cargo's v2
//! feature resolver does not hold dependencies back to that floor, and several
//! already declare more (cosmic-text and smol_str 1.89, oo7 1.92 on Linux); on
//! a toolchain near 1.85 you will most likely meet one of theirs first. A
//! recent stable is the practical answer.
//!
//! The `stitch` feature raises gpuikit's own floor to **1.95**. It is the only
//! one that does.

use gpui::App;
use rust_embed::RustEmbed;

// Core modules
pub mod element_id;
pub mod elements;
pub mod error;
pub mod fs;
pub mod icons;
pub mod input;
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
    input::bind_input_keys(cx, None);
    elements::dialog::bind_dialog_keys(cx);
    elements::toast::init(cx);
}
