//! Generic input components for GPUI applications.
//!
//! This crate provides reusable input field components with consistent
//! styling and behavior for GPUI-based applications.

mod numeric_input;
mod text_input;

pub use numeric_input::NumericInput;
pub use text_input::TextInput;
