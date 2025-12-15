/// Indicates what kind of visual focus style an element should use
///
/// - Ring: A focus ring is drawn around the element
/// - Highlight: Element is highlighted (e.g. background color change)
pub enum FocusStyle {
    Ring,
    Highlight,
}

// A trait for elements that can receive visual focus
// usually denoted by a focus ring or highlight
pub trait VisualFocus: gpui::Focusable {
    fn focus_style(&self) -> FocusStyle;
}
