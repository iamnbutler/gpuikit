//! Textarea component for multi-line text input.
//!
//! A styled wrapper around the `text_area()` element that provides form-friendly
//! styling with borders, padding, and theme colors.

use gpui::{
    div, prelude::*, px, rems, App, ElementId, Entity, EntityId, FocusHandle, Focusable,
    IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window,
};

use crate::element_id::for_entity;
use crate::elements::input::text_area;
use crate::input::InputState;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;

/// Default number of visible text rows.
const DEFAULT_ROWS: u32 = 3;

/// Approximate line height in rems for calculating min-height.
const LINE_HEIGHT_REMS: f32 = 1.5;

/// The id of the textarea backed by the `InputState` entity `state_id`.
///
/// `Textarea` is a `RenderOnce`, so nothing above it puts a per-instance
/// segment in its id path: a bare `"textarea"` would be the same id for every
/// textarea on screen. Keyed on the input state instead, which is unique per
/// textarea and stable across frames.
fn textarea_element_id(state_id: EntityId) -> ElementId {
    for_entity("textarea", state_id)
}

/// Creates a new Textarea component.
///
/// # Example
///
/// ```ignore
/// let state = cx.new(|cx| InputState::new_multiline(cx));
///
/// textarea(&state, cx)
///     .placeholder("Enter your message...")
///     .rows(4)
///     .disabled(false)
/// ```
pub fn textarea(state: &Entity<InputState>, cx: &App) -> Textarea {
    Textarea::new(state, cx)
}

/// A styled multi-line text input component.
///
/// Wraps the raw `text_area()` element with form-friendly styling including
/// borders, padding, background colors, and focus states.
#[derive(IntoElement)]
pub struct Textarea {
    state: Entity<InputState>,
    focus_handle: FocusHandle,
    placeholder: Option<SharedString>,
    rows: u32,
    disabled: bool,
    read_only: bool,
    max_height: Option<Pixels>,
    element_id: Option<ElementId>,
}

impl Textarea {
    /// Creates a new Textarea wrapping the given InputState.
    ///
    /// The InputState should be created with `InputState::new_multiline(cx)`.
    pub fn new(state: &Entity<InputState>, cx: &App) -> Self {
        Self {
            state: state.clone(),
            focus_handle: state.focus_handle(cx),
            placeholder: None,
            rows: DEFAULT_ROWS,
            disabled: false,
            read_only: false,
            max_height: None,
            element_id: None,
        }
    }

    /// Override the element id this textarea renders under.
    ///
    /// The default is derived from the `InputState` entity, which is unique
    /// and stable already. Set this only when the same state is rendered by
    /// more than one textarea in a frame; each copy then needs its own id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// The id this textarea renders under — the explicit [`Self::id`] if one
    /// was given, otherwise the state-derived default.
    pub fn element_id(&self) -> ElementId {
        self.element_id
            .clone()
            .unwrap_or_else(|| textarea_element_id(self.state.entity_id()))
    }

    /// Sets the placeholder text shown when the textarea is empty.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the number of visible text rows (affects min-height).
    ///
    /// Defaults to 3 rows.
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Sets a maximum height for the textarea.
    ///
    /// When set, the textarea will scroll vertically if content exceeds this height.
    pub fn max_height(mut self, height: impl Into<Pixels>) -> Self {
        self.max_height = Some(height.into());
        self
    }

    /// Sets the read-only state.
    ///
    /// When read-only, the textarea is visually styled to indicate it cannot be edited,
    /// but the user can still select and copy text.
    ///
    /// Note: This currently only affects visual styling. Full read-only behavior
    /// would require InputState-level support.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

impl Disableable for Textarea {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Focusable for Textarea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let element_id = self.element_id();
        let theme = cx.theme();
        let is_focused = self.focus_handle.is_focused(window);
        let disabled = self.disabled;
        let read_only = self.read_only;

        // Calculate min-height based on rows
        let min_height = rems(self.rows as f32 * LINE_HEIGHT_REMS + 1.0); // +1.0 for padding

        // Determine colors based on state
        let bg_color = if disabled {
            theme.surface_tertiary()
        } else if read_only {
            theme.surface_secondary()
        } else {
            theme.input_bg()
        };

        let border_color = if disabled {
            theme.border_subtle()
        } else if is_focused {
            theme.input_border_focused()
        } else {
            theme.input_border()
        };

        let text_color = if disabled {
            theme.fg_disabled()
        } else {
            theme.input_text()
        };

        // Build the inner text_area element
        let mut inner = text_area(&self.state, cx)
            .size_full()
            .text_color(text_color);

        if let Some(placeholder) = self.placeholder {
            inner = inner.placeholder(placeholder);
        }

        // Build the container
        div()
            .id(element_id)
            .min_h(min_height)
            .when_some(self.max_height, |this, max_h| this.max_h(max_h))
            .w_full()
            .px(rems(0.75))
            .py(rems(0.5))
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded(px(6.))
            .overflow_hidden()
            .text_sm()
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .when(!disabled && !read_only, |this| {
                this.cursor_text().when(!is_focused, |this| {
                    this.hover(|style| style.border_color(theme.input_border_hover()))
                })
            })
            .when(read_only && !disabled, |this| this.cursor_default())
            .child(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn each_textarea_renders_under_its_own_state(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let (one, two) = cx.update(|cx| {
            (
                cx.new(InputState::new_multiline),
                cx.new(InputState::new_multiline),
            )
        });

        let (first, second, overridden) = cx.update(|cx| {
            (
                textarea(&one, cx).element_id(),
                textarea(&two, cx).element_id(),
                textarea(&one, cx).id("shared-state-left").element_id(),
            )
        });

        assert_eq!(first, textarea_element_id(one.entity_id()));
        assert_ne!(first, second);
        assert_eq!(overridden, ElementId::Name("shared-state-left".into()));
    }
}
