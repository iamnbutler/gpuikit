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
pub mod editor;
pub mod element;
pub mod keymap;
pub mod keymap_ext;
pub mod syntax_highlighter;

// Internal modules
mod meta_line;

// Re-export main types
pub use buffer::{GapBuffer, TextBuffer};
pub use editor::{CursorPosition, Editor, EditorConfig};
pub use element::EditorElement;
pub use keymap::{
    ActionRegistry, ActionValue, BindingSpec, KeyBindingEntry, Keymap, KeymapCollection,
    SimpleActionRegistry,
};
pub use keymap_ext::{bind, create_bindings, BindingBuilder};
pub use meta_line::{Language, MetaLine, Selection};
pub use syntax_highlighter::SyntaxHighlighter;

// Re-export gpui for convenience
pub use gpui;
