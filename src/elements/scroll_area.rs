//! ScrollArea
//!
//! A styled scrollable container component with support for vertical and/or horizontal scrolling.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::scroll_area::scroll_area;
//!
//! // Vertical scroll area with max height
//! scroll_area("my-scroll-area")
//!     .max_h(px(300.))
//!     .vertical()
//!     .child(long_content)
//!
//! // Horizontal scroll area
//! scroll_area("horiz-scroll")
//!     .max_w(px(400.))
//!     .horizontal()
//!     .child(wide_content)
//!
//! // Both directions
//! scroll_area("both-scroll")
//!     .max_h(px(300.))
//!     .max_w(px(400.))
//!     .both()
//!     .child(large_content)
//! ```

use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, IntoElement, Length, ParentElement,
    RenderOnce, Styled, Window,
};

/// Scroll direction for the scroll area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    /// Vertical scrolling only
    #[default]
    Vertical,
    /// Horizontal scrolling only
    Horizontal,
    /// Both vertical and horizontal scrolling
    Both,
}

/// Creates a new scroll area builder.
///
/// # Arguments
///
/// * `id` - Unique identifier for the scroll area
///
/// # Example
///
/// ```ignore
/// scroll_area("my-scroll-area")
///     .max_h(px(300.))
///     .vertical()
///     .child(content)
/// ```
pub fn scroll_area(id: impl Into<ElementId>) -> ScrollArea {
    ScrollArea::new(id)
}

/// A styled scrollable container component.
///
/// Use the [`scroll_area`] function to create an instance.
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    direction: ScrollDirection,
    max_height: Option<Length>,
    max_width: Option<Length>,
    children: Vec<AnyElement>,
    full_width: bool,
    full_height: bool,
}

impl ScrollArea {
    /// Creates a new scroll area with default settings.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            direction: ScrollDirection::Vertical,
            max_height: None,
            max_width: None,
            children: Vec::new(),
            full_width: false,
            full_height: false,
        }
    }

    /// Sets the scroll direction to vertical only.
    pub fn vertical(mut self) -> Self {
        self.direction = ScrollDirection::Vertical;
        self
    }

    /// Sets the scroll direction to horizontal only.
    pub fn horizontal(mut self) -> Self {
        self.direction = ScrollDirection::Horizontal;
        self
    }

    /// Sets the scroll direction to both vertical and horizontal.
    pub fn both(mut self) -> Self {
        self.direction = ScrollDirection::Both;
        self
    }

    /// Sets the scroll direction.
    pub fn direction(mut self, direction: ScrollDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the maximum height of the scroll area.
    pub fn max_h(mut self, height: impl Into<Length>) -> Self {
        self.max_height = Some(height.into());
        self
    }

    /// Sets the maximum height in pixels.
    pub fn max_h_px(self, height: f32) -> Self {
        self.max_h(px(height))
    }

    /// Sets the maximum width of the scroll area.
    pub fn max_w(mut self, width: impl Into<Length>) -> Self {
        self.max_width = Some(width.into());
        self
    }

    /// Sets the maximum width in pixels.
    pub fn max_w_px(self, width: f32) -> Self {
        self.max_w(px(width))
    }

    /// Make the scroll area expand to fill available width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Make the scroll area expand to fill available height.
    pub fn full_height(mut self, full_height: bool) -> Self {
        self.full_height = full_height;
        self
    }

    /// Adds a child element to the scroll area.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Adds multiple child elements to the scroll area.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Build base container with id for scroll state tracking
        let container = div()
            .id(self.id)
            .flex()
            .flex_col()
            .when(self.full_width, |this| this.w_full())
            .when(self.full_height, |this| this.h_full())
            .when_some(self.max_height, |this, height| this.max_h(height))
            .when_some(self.max_width, |this, width| this.max_w(width));

        // Apply scroll behavior based on direction
        let container = match self.direction {
            ScrollDirection::Vertical => container.overflow_y_scroll().overflow_x_hidden(),
            ScrollDirection::Horizontal => container.overflow_x_scroll().overflow_y_hidden(),
            ScrollDirection::Both => container.overflow_y_scroll().overflow_x_scroll(),
        };

        // Stop scroll wheel propagation to prevent parent scrolling
        container
            .on_scroll_wheel(|_, _, cx| {
                cx.stop_propagation();
            })
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_direction_default() {
        assert_eq!(ScrollDirection::default(), ScrollDirection::Vertical);
    }

    #[test]
    fn test_scroll_area_builder() {
        let area = scroll_area("test-id").vertical();
        assert_eq!(area.direction, ScrollDirection::Vertical);

        let area = scroll_area("test-id").horizontal();
        assert_eq!(area.direction, ScrollDirection::Horizontal);

        let area = scroll_area("test-id").both();
        assert_eq!(area.direction, ScrollDirection::Both);
    }
}
