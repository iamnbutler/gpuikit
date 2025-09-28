//! Generic input components for GPUI applications.
//!
//! This crate provides reusable input field components with consistent
//! styling and behavior for GPUI-based applications.

mod input_handler;
mod text_input;

pub use input_handler::InputHandler;
pub use text_input::TextInput;
