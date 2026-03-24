//! ButtonGroup component for gpuikit
//!
//! A container component that groups multiple buttons together, visually
//! connecting them and handling proper border radius on the edges.

use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, IntoElement, ParentElement, Rems,
    RenderOnce, Styled, Window,
};

/// Convenience function to create a button group
pub fn button_group() -> ButtonGroup {
    ButtonGroup::new()
}

/// Orientation for the button group layout
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A container component that groups multiple buttons together
#[derive(IntoElement)]
pub struct ButtonGroup {
    children: Vec<AnyElement>,
    orientation: ButtonGroupOrientation,
    attached: bool,
    gap: Rems,
}

impl ButtonGroup {
    pub fn new() -> Self {
        ButtonGroup {
            children: Vec::new(),
            orientation: ButtonGroupOrientation::default(),
            attached: false,
            gap: rems(0.25),
        }
    }

    /// Add a single child element to the group
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple children to the group
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }

    /// Set the orientation to horizontal (default)
    pub fn horizontal(mut self) -> Self {
        self.orientation = ButtonGroupOrientation::Horizontal;
        self
    }

    /// Set the orientation to vertical
    pub fn vertical(mut self) -> Self {
        self.orientation = ButtonGroupOrientation::Vertical;
        self
    }

    /// Set the orientation
    pub fn orientation(mut self, orientation: ButtonGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Enable attached/flush mode (no gap between buttons)
    pub fn attached(mut self, attached: bool) -> Self {
        self.attached = attached;
        self
    }

    /// Set custom gap between buttons (ignored when attached)
    pub fn gap(mut self, gap: Rems) -> Self {
        self.gap = gap;
        self
    }
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_horizontal = self.orientation == ButtonGroupOrientation::Horizontal;
        let child_count = self.children.len();

        let container = if is_horizontal {
            div().flex().flex_row()
        } else {
            div().flex().flex_col()
        };

        // Apply gap only when not attached
        let container = if self.attached {
            container
        } else {
            container.gap(self.gap)
        };

        // When attached, use overflow hidden to clip children and apply border radius to container
        let container = if self.attached && child_count > 0 {
            container.overflow_hidden().rounded(rems(0.25))
        } else {
            container
        };

        container.children(
            self.children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    let is_first = index == 0;
                    let is_last = index == child_count - 1;

                    // Wrap each child in a div to control border radius when attached
                    if self.attached {
                        let wrapper = div().overflow_hidden();

                        let wrapper = if is_horizontal {
                            // Horizontal: first gets left radius, last gets right radius
                            wrapper
                                .when(is_first, |w| {
                                    w.rounded_tl(rems(0.25)).rounded_bl(rems(0.25))
                                })
                                .when(is_last, |w| {
                                    w.rounded_tr(rems(0.25)).rounded_br(rems(0.25))
                                })
                                .when(!is_first && !is_last, |w| w.rounded_none())
                        } else {
                            // Vertical: first gets top radius, last gets bottom radius
                            wrapper
                                .when(is_first, |w| {
                                    w.rounded_tl(rems(0.25)).rounded_tr(rems(0.25))
                                })
                                .when(is_last, |w| {
                                    w.rounded_bl(rems(0.25)).rounded_br(rems(0.25))
                                })
                                .when(!is_first && !is_last, |w| w.rounded_none())
                        };

                        wrapper.child(child).into_any_element()
                    } else {
                        // Non-attached mode: just render the child as-is
                        child
                    }
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_group_creation() {
        let group = button_group();
        assert_eq!(group.orientation, ButtonGroupOrientation::Horizontal);
        assert!(!group.attached);
        assert!(group.children.is_empty());
    }

    #[test]
    fn test_button_group_vertical() {
        let group = button_group().vertical();
        assert_eq!(group.orientation, ButtonGroupOrientation::Vertical);
    }

    #[test]
    fn test_button_group_attached() {
        let group = button_group().attached(true);
        assert!(group.attached);
    }
}
