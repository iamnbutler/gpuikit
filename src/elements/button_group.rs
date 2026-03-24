//! ButtonGroup component for gpuikit
//!
//! A component for grouping related buttons together with connected borders.

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use crate::traits::orientable::{Orientable, Orientation};
use gpui::{
    div, rems, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, Window,
};

/// Create a new button group with the given ID.
pub fn button_group(id: impl Into<ElementId>) -> ButtonGroup {
    ButtonGroup::new(id)
}

/// A component for grouping related buttons together.
///
/// Buttons in a group are visually connected with shared borders.
/// The first and last buttons have rounded corners on their outer edges.
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    children: Vec<AnyElement>,
    orientation: Orientation,
    disabled: bool,
}

impl ButtonGroup {
    /// Create a new button group with the given ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            orientation: Orientation::Horizontal,
            disabled: false,
        }
    }

    /// Add a child element to the button group.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple children to the button group.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let child_count = self.children.len();
        let is_horizontal = self.orientation == Orientation::Horizontal;

        let mut container = div()
            .id(self.id)
            .flex()
            .flex_none()
            .overflow_hidden()
            .border_1()
            .border_color(theme.button_border())
            .rounded(rems(0.25));

        container = if is_horizontal {
            container.flex_row()
        } else {
            container.flex_col()
        };

        if self.disabled {
            container = container.opacity(0.65).cursor_not_allowed();
        }

        container.children(self.children.into_iter().enumerate().map(|(index, child)| {
            let is_first = index == 0;
            let is_last = index == child_count - 1;

            let mut wrapper = div()
                .flex()
                .items_center()
                .justify_center()
                .bg(theme.button_bg())
                .text_color(theme.fg());

            // Add internal borders between buttons
            if !is_last {
                wrapper = if is_horizontal {
                    wrapper.border_r_1().border_color(theme.button_border())
                } else {
                    wrapper.border_b_1().border_color(theme.button_border())
                };
            }

            // Rounded corners for first/last items
            if is_horizontal {
                if is_first {
                    wrapper = wrapper.rounded_l(rems(0.2));
                }
                if is_last {
                    wrapper = wrapper.rounded_r(rems(0.2));
                }
            } else {
                if is_first {
                    wrapper = wrapper.rounded_t(rems(0.2));
                }
                if is_last {
                    wrapper = wrapper.rounded_b(rems(0.2));
                }
            }

            wrapper.child(child)
        }))
    }
}

impl Orientable for ButtonGroup {
    fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl Disableable for ButtonGroup {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
