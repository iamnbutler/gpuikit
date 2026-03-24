//! ScrollArea component for styled scrollable containers
//!
//! Provides a wrapper around gpui's native scroll functionality with
//! consistent styling and optional theme-aware scrollbar customization.
//!
//! # Example
//!
//! ```ignore
//! scroll_area("my-scroll")
//!     .max_h(px(300.))
//!     .vertical()
//!     .child(long_content)
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, Window,
};

/// Scroll direction configuration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Scroll vertically only (default)
    #[default]
    Vertical,
    /// Scroll horizontally only
    Horizontal,
    /// Scroll in both directions
    Both,
}

/// A styled scrollable container component.
///
/// Wraps gpui's native scroll functionality with consistent styling
/// and theme-aware scrollbar colors.
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    direction: ScrollDirection,
    max_height: Option<Pixels>,
    max_width: Option<Pixels>,
    child: Option<AnyElement>,
}

impl ScrollArea {
    /// Create a new scroll area with the given id and default vertical scrolling.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            direction: ScrollDirection::Vertical,
            max_height: None,
            max_width: None,
            child: None,
        }
    }

    /// Set the scroll direction to vertical only.
    pub fn vertical(mut self) -> Self {
        self.direction = ScrollDirection::Vertical;
        self
    }

    /// Set the scroll direction to horizontal only.
    pub fn horizontal(mut self) -> Self {
        self.direction = ScrollDirection::Horizontal;
        self
    }

    /// Enable scrolling in both directions.
    pub fn both(mut self) -> Self {
        self.direction = ScrollDirection::Both;
        self
    }

    /// Set the scroll direction explicitly.
    pub fn direction(mut self, direction: ScrollDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the maximum height for the scroll area.
    ///
    /// Content taller than this will be scrollable.
    pub fn max_h(mut self, height: impl Into<Pixels>) -> Self {
        self.max_height = Some(height.into());
        self
    }

    /// Set the maximum width for the scroll area.
    ///
    /// Content wider than this will be scrollable.
    pub fn max_w(mut self, width: impl Into<Pixels>) -> Self {
        self.max_width = Some(width.into());
        self
    }

    /// Set the child element to display in the scroll area.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Start with an interactive div (id required for scroll methods)
        let mut container = div().id(self.id).flex().flex_col();

        // Apply scroll behavior based on direction
        container = match self.direction {
            ScrollDirection::Vertical => container.overflow_y_scroll(),
            ScrollDirection::Horizontal => container.overflow_x_scroll(),
            ScrollDirection::Both => container.overflow_scroll(),
        };

        // Apply max constraints
        if let Some(max_h) = self.max_height {
            container = container.max_h(max_h);
        }

        if let Some(max_w) = self.max_width {
            container = container.max_w(max_w);
        }

        // Prevent scroll events from propagating to parent scrollable areas
        container = container.on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        });

        // Apply theme-aware styling (subtle, non-intrusive background)
        container = container.bg(theme.surface());

        // Add child content
        if let Some(child) = self.child {
            container = container.child(child);
        }

        container
    }
}

/// Create a new scroll area with default vertical scrolling.
///
/// # Arguments
///
/// * `id` - Unique identifier for the scroll area element
///
/// # Example
///
/// ```ignore
/// scroll_area("my-scroll")
///     .max_h(px(300.))
///     .child(long_list_content)
/// ```
pub fn scroll_area(id: impl Into<ElementId>) -> ScrollArea {
    ScrollArea::new(id)
}
