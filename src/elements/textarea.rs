//! Textarea component for multi-line text input.
//!
//! A styled wrapper around the `text_area()` element that provides form-friendly
//! styling with borders, padding, and theme colors.

use gpui::{
    div, prelude::*, App, ElementId, Entity, EntityId, FocusHandle, Focusable, IntoElement,
    ParentElement, Pixels, Rems, RenderOnce, SharedString, Styled, Window,
};

use crate::element_id::for_entity;
use crate::elements::input::{disabled_display, text_area};
use crate::input::InputState;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;

/// Default number of visible text rows.
const DEFAULT_ROWS: u32 = 3;

/// The 1px border the textarea draws, in rems at a 16px root — the one part of
/// its height that is not a multiple of the line box.
const BORDER_REMS: f32 = 1.0 / 16.0;

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
    /// `None` means "say nothing about read-only" — see [`Textarea::read_only`].
    read_only: Option<bool>,
    max_height: Option<Pixels>,
    element_id: Option<ElementId>,
    size: ControlSize,
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
            read_only: None,
            max_height: None,
            element_id: None,
            size: ControlSize::default(),
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

    /// Sets the read-only state: the textarea refuses every user edit —
    /// typing, IME composition, paste, cut's removal, the delete family, tab,
    /// newlines, undo and redo — while keeping focus, cursor movement,
    /// selection, copy and scrolling.
    ///
    /// **This writes through to the `InputState` it was given**, at the top of
    /// `render`. A wrapper-level property can only be enforced by the layer
    /// that owns the keystrokes, and that is the state. It is scoped as
    /// tightly as it can be: only when the caller calls this, and
    /// `InputState::set_read_only` no-ops when nothing changed. A textarea
    /// that never calls it says nothing, and a state made read-only by its
    /// owner is never quietly handed back.
    ///
    /// For a control that also refuses *focus*, use
    /// [`disabled`](Disableable::disabled) — read-only cannot implement that,
    /// because an element that is painted is in the tab order.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
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

impl ControlSized for Textarea {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
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

        // Imposed before anything reads it back, so the chrome below and the
        // enforcement in `InputState` cannot disagree within one frame. A
        // wrapper that says nothing leaves a state its owner made read-only
        // alone.
        if let Some(read_only) = self.read_only {
            self.state
                .update(cx, |state, cx| state.set_read_only(read_only, cx));
        }
        let read_only = self.state.read(cx).is_read_only();

        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let is_focused = self.focus_handle.is_focused(window);
        let disabled = self.disabled;

        // The height is the rows the caller asked for, plus the padding and
        // border that sit around them — rather than the old
        // `rows * 1.5 + 1.0`, whose `+1.0` was commented "for padding" and
        // matched no padding the element actually drew.
        //
        // The rung's `padding_y` is sized to centre one line box inside one
        // rung, which is far too tight around several lines; a textarea is a
        // box of text rather than a control with a line in it, so it uses the
        // rung's padding on both axes.
        let padding = metrics.padding_x;
        let min_height = metrics.multiline_line_height() * self.rows as f32
            + (padding + Rems(BORDER_REMS)) * 2.0;

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

        // A disabled textarea paints *no live element at all*, only static
        // text. Dimming a live `text_area()` — which is what `disabled` used
        // to do — leaves a control that looks inert and still takes focus,
        // keystrokes and IME input, because a painted `Input` registers its
        // actions and its IME handler and is in the tab order. Read-only
        // cannot substitute for the same reason.
        //
        // Consequence, deliberately: a disabled textarea *clips* a value
        // longer than its rows rather than scrolling it. `read_only` is the
        // option that keeps scrolling.
        let content = if disabled {
            let value = self.state.read(cx).content();
            let (text, is_placeholder) = disabled_display(value, self.placeholder.as_ref());
            let color = if is_placeholder {
                theme.input_placeholder()
            } else {
                text_color
            };

            div()
                .size_full()
                .overflow_hidden()
                // A live `Input` contributes no content height — its
                // `request_layout` has no children — so the box below is
                // exactly `min_h`. Static text would push it out to fit the
                // value, and toggling `disabled` would resize the form.
                .line_height(metrics.multiline_line_height())
                .text_color(color)
                .child(text)
                .into_any_element()
        } else {
            let mut inner = text_area(&self.state, cx)
                .control_size(self.size)
                .size_full()
                .text_color(text_color);

            if let Some(placeholder) = self.placeholder {
                inner = inner.placeholder(placeholder);
            }

            inner.into_any_element()
        };

        // Build the container
        div()
            .id(element_id)
            // A fixed height when disabled, for the reason above: static text
            // grows the box, a live element does not.
            .map(|this| {
                if disabled {
                    this.h(min_height)
                } else {
                    this.min_h(min_height)
                }
            })
            .when_some(self.max_height, |this, max_h| this.max_h(max_h))
            .w_full()
            .p(padding)
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded(metrics.radius)
            .overflow_hidden()
            .text_size(metrics.text_size)
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .when(!disabled && !read_only, |this| {
                this.cursor_text().when(!is_focused, |this| {
                    this.hover(|style| style.border_color(theme.input_border_hover()))
                })
            })
            // Read-only keeps the I-beam — its text is still selectable — and
            // drops the hover border, which invites an edit.
            .when(read_only && !disabled, |this| this.cursor_text())
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::input::tests::focused_input_window;
    use gpui::TestAppContext;

    /// Draw one textarea, focused, in its own window, and type `keystrokes`
    /// into it. Reports what the state holds afterwards.
    ///
    /// No assertion anywhere below is about opacity: the question is whether a
    /// keystroke lands, which is the thing that was broken.
    fn type_into(
        cx: &mut TestAppContext,
        build: impl Fn(&Entity<InputState>, &App) -> Textarea + 'static,
        content: &str,
        keystrokes: &str,
    ) -> String {
        let state = cx.update(|cx| cx.new(InputState::new_multiline));
        let content = content.to_string();
        state.update(cx, |state, cx| state.set_content(content, cx));

        let for_render = state.clone();
        let cx = focused_input_window(cx, &state, move |_window, cx| {
            build(&for_render, cx).into_any_element()
        });

        // Without this the keystroke lands on the previous frame.
        cx.run_until_parked();
        cx.simulate_keystrokes(keystrokes);
        cx.run_until_parked();

        state.read_with(cx, |state, _| state.content().to_string())
    }

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

    /// The reported bug: `disabled(true)` set an opacity over a fully live
    /// `text_area()`, so the control looked inert and still took keystrokes.
    #[gpui::test]
    fn a_disabled_textarea_does_not_take_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(
            cx,
            |state, cx| textarea(state, cx).disabled(true),
            "kept",
            "x",
        );

        assert_eq!(after, "kept");
    }

    /// `read_only(true)` was the same lie with different colours, and said so
    /// in its own doc comment.
    #[gpui::test]
    fn a_read_only_textarea_does_not_take_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(
            cx,
            |state, cx| textarea(state, cx).read_only(true),
            "kept",
            "x",
        );

        assert_eq!(after, "kept");
    }

    /// The check that the harness bites. `set_content` resets the cursor to
    /// offset 0, so the typed character lands in front.
    #[gpui::test]
    fn an_editable_textarea_takes_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(cx, |state, cx| textarea(state, cx), "kept", "x");

        assert_eq!(after, "xkept");
    }

    /// A wrapper that says nothing about read-only must not hand a state its
    /// owner made read-only back to the user. This is why the wrapper field is
    /// an `Option<bool>` rather than a `bool` defaulting to `false`.
    #[gpui::test]
    fn a_plain_textarea_does_not_clear_a_read_only_state(cx: &mut TestAppContext) {
        let state = cx.update(|cx| cx.new(|cx| InputState::new_multiline(cx).read_only(true)));
        state.update(cx, |state, cx| state.set_content("kept", cx));

        let for_render = state.clone();
        let cx = focused_input_window(cx, &state, move |_window, cx| {
            textarea(&for_render, cx).into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("x");
        cx.run_until_parked();

        state.read_with(cx, |state, _| {
            assert!(state.is_read_only(), "the wrapper cleared the state's flag");
            assert_eq!(state.content(), "kept");
        });
    }

    /// And an explicit `false` does lift it — the flag is imposed, not merely
    /// set once.
    #[gpui::test]
    fn read_only_false_lifts_a_read_only_state(cx: &mut TestAppContext) {
        let state = cx.update(|cx| cx.new(|cx| InputState::new_multiline(cx).read_only(true)));
        state.update(cx, |state, cx| state.set_content("kept", cx));

        let for_render = state.clone();
        let cx = focused_input_window(cx, &state, move |_window, cx| {
            textarea(&for_render, cx)
                .read_only(false)
                .into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("x");
        cx.run_until_parked();

        state.read_with(cx, |state, _| {
            assert!(!state.is_read_only());
            assert_eq!(state.content(), "xkept");
        });
    }
}
