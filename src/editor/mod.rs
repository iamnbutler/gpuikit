//! A standalone editor component for GPUI
//!
//! This crate provides a text editor widget for GPUI applications with syntax highlighting support.
//!
//! # Architecture
//!
//! The editor is structured in three layers:
//!
//! - **Editor**: The core data model and editing operations
//! - **EditorElement**: The GPUI element that renders an Editor
//! - **EditorView**: A complete view with keyboard handling (see examples)

pub mod buffer;
// `editor::editor` is a public path; renaming the module is a semver break,
// not a lint fix.
#[allow(clippy::module_inception)]
pub mod editor;
pub mod element;
mod tests;

pub mod syntax_highlighter;
pub mod view;

// Internal modules
mod meta_line;

// Re-export main types
pub use buffer::{GapBuffer, TextBuffer};
pub use editor::{CursorPosition, Editor, EditorConfig};
pub use element::EditorElement;
// Re-export keymap types from keymap module
pub use crate::keymap::extensions::{BindingBuilder, bind, create_bindings};
pub use crate::keymap::{BindingSpec, Keymap, KeymapCollection};
pub use meta_line::{Language, MetaLine, Selection};
pub use syntax_highlighter::SyntaxHighlighter;
pub use view::{EditorBindings, EditorView, bind_editor_keys};

// Re-export gpui for convenience
pub use gpui;
