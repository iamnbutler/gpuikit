#![allow(missing_docs)]
//! gpuikit
//!
//! A comprehensive UI component library for GPUI applications.
//!
//! # Quick Start
//!
//! ```no_run
//! use gpui::Application;
//! use gpuikit::init;
//!
//! fn main() {
//!     Application::with_platform(gpui_platform::current_platform(false))
//!         .with_assets(gpuikit::assets())
//!         .run(|cx| {
//!             init(cx);
//!             // ... your app code
//!         });
//! }
//! ```
//!
//! The platform comes from `gpui_platform`, which your app depends on
//! alongside `gpui` — gpuikit does not re-export either. `false` asks for a
//! real windowing platform rather than a headless one.
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
//! own source — async closures, and edition 2024 — rather than a guarantee
//! about a whole build. Edition 2024 selects cargo's v3 resolver, which unlike
//! v2 does take that floor into account: when it picks a *new* version of a
//! dependency it prefers one whose own `rust-version` fits. That is a
//! preference, not a wall, and it says nothing about the versions `Cargo.lock`
//! already names — several of those declare more (cosmic-text and smol_str
//! 1.89, oo7 1.92 on Linux), so on a toolchain near 1.85 you will most likely
//! meet one of theirs first. A recent stable is the practical answer.
//!
//! The `stitch` feature raises gpuikit's own floor to **1.95**. It is the only
//! one that does.

use gpui::App;
use rust_embed::RustEmbed;

// Core modules
pub mod a11y;
pub mod date;
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
pub mod selection;
pub mod theme;
pub mod traits;
pub mod utils;

// Feature-gated editor module
#[cfg(feature = "editor")]
pub mod editor;

pub use icons::Icons as DefaultIcons;

/// Tests for the release workflows' version guard — that the version either of
/// them would publish is the one `CHANGELOG.md` names. `release.yml` computes
/// the version; `release-deploy.yml` is the one that runs `cargo publish`, and
/// it is reachable without `release.yml` having run at all, so both carry it.
///
/// No runtime code, and nothing outside a test build: the module exists
/// because `cargo test --lib` is the only thing in this repository that can
/// check a workflow before it runs for real. See its own docs.
#[cfg(test)]
mod release_version_guard;

/// Tests for the rule that keeps a workflow's outside values out of its shell
/// — no `${{ }}` inside a `run:` body, and free-form values judged in a step of
/// their own before anything uses them.
///
/// No runtime code, and nothing outside a test build. Covers every workflow in
/// `.github/workflows/`, which a test enforces by reading the directory. See
/// its own docs.
#[cfg(test)]
mod release_input_validation;

/// Tests for the build configuration that keeps `ld` from being OOM-killed
/// while linking this crate's eight examples — the dev profile's debug level,
/// Linux's `split-debuginfo`, and the `examples` feature every `[[example]]`
/// requires.
///
/// No runtime code, and nothing outside a test build. One test reads
/// `examples/` from disk, because a new undeclared file there is autodiscovered
/// as a target and cannot carry `required-features`. See its own docs.
#[cfg(test)]
mod build_profile_guard;

/// Tests for the rule that this crate creates no thread it cannot join — no
/// `smol` / `async-io` dependency, no `smol::` or `async_io::` in the source,
/// and the two delays (cursor blink, toast auto-dismiss) scheduled on gpui's
/// `BackgroundExecutor::timer`.
///
/// No runtime code, and nothing outside a test build. The `async-io` thread's
/// `main_loop` has no exit path, so it raced process teardown and aborted a
/// fully green `cargo test --lib` (#190). See its own docs.
#[cfg(test)]
mod undying_thread_guard;

/// Tests for the rule that a rustdoc example which is not checked does not
/// exist — no `` ```ignore `` anywhere in `src/`, and `no_run` only with a
/// reason on record. rustdoc never compiles an `ignore`d block, so the crate's
/// own Quick Start went on naming `Application::new()` long after that
/// function stopped existing.
///
/// No runtime code, and nothing outside a test build. Its docs carry the
/// hidden prelude a new example should copy.
#[cfg(test)]
mod doctest_fence_guard;

/// Tests for the rule that runtime code stays runnable on
/// `wasm32-unknown-unknown` — no `std::time::Instant`/`SystemTime`,
/// `std::fs`, or `std::thread::spawn` outside an explicit allowlist of
/// native-only APIs and test-only code. These compile for wasm and then
/// panic or error in the browser, which no local `cargo test` would catch.
///
/// No runtime code, and nothing outside a test build. See its own docs, and
/// <https://github.com/iamnbutler/gpuikit-demo> for gpuikit running on wasm.
#[cfg(test)]
mod wasm_compat_guard;

/// Embedded assets for gpuikit (icons, fonts, etc.)
#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

/// Returns the gpuikit asset source, for `Application::with_assets`.
///
/// # Example
/// ```no_run
/// # use gpui::Application;
/// Application::with_platform(gpui_platform::current_platform(false))
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
    // Before `bind_input_keys`, and the order is load-bearing: both bind Tab,
    // gpui prefers the later-registered binding at equal context depth, and
    // that is what keeps Tab inside a focused text input rather than moving
    // focus out of it. See `a11y`'s module docs, section 4.
    a11y::bind_focus_keys(cx);
    input::bind_input_keys(cx, None);
    elements::dialog::bind_dialog_keys(cx);
    // After `bind_focus_keys`, and after `bind_dialog_keys`. Binding
    // precedence is by key-context depth with ties broken by registration
    // order, and a binding with no context counts as the deepest — so these
    // `Listbox`-scoped bindings can never outrank `a11y`'s context-less Tab
    // (the popup answers Tab with an action listener instead) and always
    // outrank `Dialog`'s Escape, which is what lets a select inside a dialog
    // close its own popup. Registering last is belt and braces for the second
    // half of that. See `elements::select`'s `# The keyboard`.
    elements::select::bind_select_keys(cx);
    // Same reasoning as the listbox above, one component along: the grid's
    // arrows, Home/End, PageUp/PageDown and Enter are bound in the deeper
    // `Calendar` context, so a calendar inside a dialog keeps them and the
    // dialog still gets Escape. See `elements::calendar`'s `# The keyboard`.
    elements::calendar::bind_calendar_keys(cx);
    // **After `input::bind_input_keys`, and the order is load-bearing.** Both
    // of these bind `up`, `down`, `enter` and `escape` in a
    // `"<Component> > Input"` context predicate, which matches at the focused
    // field's own node and so *ties* on depth with `bind_input_keys`' plain
    // `Input` binding. gpui's `KeyBindingContextPredicate::Descendant`
    // (`gpui/src/keymap/context.rs:181`, parsed from `>` at `:361`) is what
    // makes that predicate legal, and `Keymap::bindings_for_input`
    // (`gpui/src/keymap.rs:173`) sorts candidates
    // `depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))` — descending depth, then
    // descending registration index. So the later registration wins the tie,
    // and registered before `bind_input_keys` these two would compile, run,
    // and do nothing: every arrow key would move the text cursor. See
    // `elements::combobox`'s and `elements::command`'s `# The keyboard`.
    elements::combobox::bind_combobox_keys(cx);
    elements::command::bind_command_keys(cx);
    elements::toast::init(cx);
    // The editor binds its keys in its own `Editor` context, distinct from
    // `Input`, so it neither ties with nor outranks anything above — order is
    // not load-bearing here. Only present with the feature that compiles it.
    #[cfg(feature = "editor")]
    editor::bind_editor_keys(cx, None);
}
