//! A single-line text field: one bordered box that owns its chrome, with
//! optional adornments laid inside it.
//!
//! `TextField` is the single-line counterpart to [`Textarea`]. It replaces
//! `InputGroup`, which drew an addon cell, a stripped input and another addon
//! cell as three sibling boxes and then spent most of its code disguising them
//! as one — suppressing radii per side, stripping a border the raw `Input`
//! never had, and hiding the seams between the three. There is one box here,
//! so nothing has to be hidden.
//!
//! # Why the adornments live here and not on `Input`
//!
//! `input()` is a hand-written [`Element`] that paints text and
//! nothing else: no border, no background, no focus ring, no notion of
//! disabled, no intrinsic size. Folding adornments into it would mean teaching
//! that element — text layout, IME, mouse-to-text-position mapping — to lay
//! out and paint children and to shrink its own text bounds, and it would
//! inherit no chrome in return, because there is none. Chrome around a raw
//! input is the pattern the crate already has: [`Textarea`] is chrome around
//! `text_area()`, and this is the same thing around `input()`.
//!
//! # Example
//!
//! ```
//! # use gpui::{Context, Entity, IntoElement, Render, Window, prelude::*};
//! use gpuikit::DefaultIcons;
//! use gpuikit::elements::text_field::{Adornment, text_field};
//! # use gpuikit::input::InputState;
//! # struct D { state: Entity<InputState> }
//! # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let state = self.state.clone();
//! # let _ =
//! text_field(&state, cx)
//!     .placeholder("Search")
//!     .prefix(Adornment::icon(DefaultIcons::magnifying_glass()))
//! # ;
//!
//! text_field(&state, cx)
//!     .prefix(Adornment::text("https://"))
//!     .suffix(Adornment::text(".com"))
//! # }}
//! # let mut tcx = gpui::TestAppContext::single();
//! # tcx.update(gpuikit::init);
//! # let _ = tcx.add_window_view(|_, cx| D { state: cx.new(InputState::new_singleline) });
//! ```
//!
//! A button that acts on the field is composition, not a field feature:
//!
//! ```
//! # use gpui::{Context, Entity, IntoElement, Render, Window, prelude::*};
//! use gpuikit::elements::button::button;
//! use gpuikit::elements::text_field::text_field;
//! use gpuikit::layout::h_stack;
//! # use gpuikit::input::InputState;
//! # struct D { state: Entity<InputState> }
//! # impl Render for D { fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//! # let state = self.state.clone();
//! h_stack()
//!     .gap_2()
//!     .child(text_field(&state, cx))
//!     .child(button("go", "Go"))
//! # }}
//! # let mut tcx = gpui::TestAppContext::single();
//! # tcx.update(gpuikit::init);
//! # let _ = tcx.add_window_view(|_, cx| D { state: cx.new(InputState::new_singleline) });
//! ```
//!
//! [`Textarea`]: crate::elements::textarea::Textarea

use gpui::{
    AnyElement, App, ElementId, Entity, EntityId, FocusHandle, Focusable, IntoElement,
    ParentElement, Rems, RenderOnce, SharedString, Styled, Svg, Window, div, prelude::*,
};

use crate::element_id::for_entity;
use crate::elements::input::{disabled_display, input};
use crate::input::InputState;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;

/// The width the text itself will not shrink below.
///
/// A single-line `input()` has no intrinsic size — `request_layout` passes the
/// style through with no children — so without a floor the text area of the
/// field collapses to nothing next to its adornments. This is a floor, not a
/// preference: a `full_width` field inside a parent narrower than this plus
/// its adornments will overflow that parent.
const MIN_CONTENT_WIDTH: Rems = Rems(8.0);

/// The id of the field backed by the `InputState` entity `state_id`.
///
/// `TextField` is a `RenderOnce`, so nothing above it puts a per-instance
/// segment in its id path: a bare `"text-field"` would be the same id for
/// every field on screen. Keyed on the input state instead, which is unique
/// per field and stable across frames. See [`crate::element_id`].
fn text_field_element_id(state_id: EntityId) -> ElementId {
    for_entity("text-field", state_id)
}

/// Creates a new [`TextField`] wrapping the given input state.
///
/// The `InputState` should be created with `InputState::new_singleline(cx)`.
pub fn text_field(state: &Entity<InputState>, cx: &App) -> TextField {
    TextField::new(state, cx)
}

/// Something laid inside the field's border, before or after the text.
///
/// Icon and text adornments take the field's muted foreground and follow its
/// disabled state. [`Adornment::element`] is the escape hatch and is passed
/// through untouched — a clear button, a loading indicator. It has to fit: the
/// field clips overflow, so size it against the rung (an `icon_button` at the
/// same rung, or `.box_size()` down to the field's ink).
pub struct Adornment(AdornmentKind);

enum AdornmentKind {
    // Boxed: a gpui `Svg` is over a kilobyte, which would otherwise set the
    // size of every adornment including a one-word label.
    Icon(Box<Svg>),
    Text(SharedString),
    Element(AnyElement),
}

impl Adornment {
    /// An icon, sized and coloured by the field.
    pub fn icon(icon: Svg) -> Self {
        Adornment(AdornmentKind::Icon(Box::new(icon)))
    }

    /// A short text label — a unit, a scheme, a currency symbol.
    pub fn text(text: impl Into<SharedString>) -> Self {
        Adornment(AdornmentKind::Text(text.into()))
    }

    /// Any element, passed through untouched.
    pub fn element(element: impl IntoElement) -> Self {
        Adornment(AdornmentKind::Element(element.into_any_element()))
    }
}

/// A single-line text field: one bordered box that owns the border, the
/// background, the radius, the hover/focus/disabled states and the padding,
/// with optional adornments laid inside it.
#[derive(IntoElement)]
pub struct TextField {
    state: Entity<InputState>,
    focus_handle: FocusHandle,
    placeholder: Option<SharedString>,
    prefix: Option<Adornment>,
    suffix: Option<Adornment>,
    full_width: bool,
    disabled: bool,
    /// `None` means "say nothing about read-only" — see [`TextField::read_only`].
    read_only: Option<bool>,
    size: ControlSize,
    element_id: Option<ElementId>,
}

impl TextField {
    /// Creates a new field wrapping the given `InputState`.
    pub fn new(state: &Entity<InputState>, cx: &App) -> Self {
        Self {
            state: state.clone(),
            focus_handle: state.focus_handle(cx),
            placeholder: None,
            prefix: None,
            suffix: None,
            full_width: false,
            disabled: false,
            read_only: None,
            size: ControlSize::default(),
            element_id: None,
        }
    }

    /// Override the element id this field renders under.
    ///
    /// The default is derived from the `InputState` entity, which is unique
    /// and stable already. Set this only when the same state is rendered by
    /// more than one field in a frame; each copy then needs its own id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// The id this field renders under — the explicit [`Self::id`] if one was
    /// given, otherwise the state-derived default.
    pub fn element_id(&self) -> ElementId {
        self.element_id
            .clone()
            .unwrap_or_else(|| text_field_element_id(self.state.entity_id()))
    }

    /// Sets the placeholder text shown when the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// An adornment before the text.
    pub fn prefix(mut self, adornment: Adornment) -> Self {
        self.prefix = Some(adornment);
        self
    }

    /// An adornment after the text.
    pub fn suffix(mut self, adornment: Adornment) -> Self {
        self.suffix = Some(adornment);
        self
    }

    /// Make the field expand to fill the width available to it.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Sets the read-only state: the field refuses every user edit — typing,
    /// IME composition, paste, cut's removal, the delete family, tab, undo and
    /// redo — while keeping focus, cursor movement, selection and copy.
    ///
    /// **This writes through to the `InputState` it was given**, at the top of
    /// `render`, for the same reason and with the same scoping as
    /// [`Textarea::read_only`](crate::elements::textarea::Textarea::read_only).
    ///
    /// Without this, `InputState::set_read_only` on a field would produce a
    /// control that refuses edits while looking exactly like one that takes
    /// them — the inverse of the bug `disabled` had.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }
}

impl Disableable for TextField {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for TextField {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Focusable for TextField {
    /// The field's focus handle *is* the state's, so focusing the field and
    /// focusing the text it wraps are the same act.
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let element_id = self.element_id();

        // Imposed before anything reads it back, so the chrome and the
        // enforcement in `InputState` cannot disagree within one frame.
        if let Some(read_only) = self.read_only {
            self.state
                .update(cx, |state, cx| state.set_read_only(read_only, cx));
        }
        let read_only = self.state.read(cx).is_read_only();

        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let is_focused = self.focus_handle.is_focused(window);
        let disabled = self.disabled;

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

        let content = if disabled {
            // A disabled field renders its value as static text. This is the
            // only layer that can honour `disabled` at all: the raw `Input`
            // registers its actions and its IME handler whenever it is painted
            // and is in the tab order — which is why the old `InputGroup`,
            // which only dimmed a live input, still took keystrokes.
            // `InputState`'s read-only support closes every editing path but
            // cannot close focus, so it is the answer to `read_only`, not to
            // `disabled`.
            let value = self.state.read(cx).content();
            let (text, is_placeholder) = disabled_display(value, self.placeholder.as_ref());
            let color = if is_placeholder {
                theme.input_placeholder()
            } else {
                theme.fg_disabled()
            };

            div()
                .flex_1()
                .min_w(MIN_CONTENT_WIDTH)
                .overflow_hidden()
                .whitespace_nowrap()
                .text_color(color)
                .child(text)
                .into_any_element()
        } else {
            let mut inner = input(&self.state, cx)
                .control_size(self.size)
                .flex_1()
                .h_full()
                .min_w(MIN_CONTENT_WIDTH)
                .text_color(theme.input_text());

            if let Some(placeholder) = self.placeholder {
                inner = inner.placeholder(placeholder);
            }

            inner.into_any_element()
        };

        let focus_handle = self.focus_handle.clone();

        h_stack()
            .id(element_id)
            .items_center()
            .h(metrics.height)
            .gap(metrics.gap)
            .px(metrics.padding_x)
            .when(!self.full_width, |this| this.flex_none())
            .when(self.full_width, |this| this.w_full())
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded(metrics.radius)
            .overflow_hidden()
            .text_size(metrics.text_size)
            .line_height(metrics.line_height)
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .when(!disabled, |this| {
                this.cursor_text()
                    // A hover border invites an edit a read-only field will
                    // not take. The I-beam stays: the text is still
                    // selectable.
                    .when(!is_focused && !read_only, |this| {
                        this.hover(|style| style.border_color(theme.input_border_hover()))
                    })
                    // The whole box focuses the text, including the padding
                    // and the adornments — clicking a clear button and
                    // carrying on typing is the wanted behaviour. An
                    // adornment that should *not* focus has to stop
                    // propagation itself.
                    .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                        window.focus(&focus_handle, cx);
                    })
            })
            .when_some(self.prefix, |this, adornment| {
                this.child(render_adornment(adornment, disabled, metrics.ink, cx))
            })
            .child(content)
            .when_some(self.suffix, |this, adornment| {
                this.child(render_adornment(adornment, disabled, metrics.ink, cx))
            })
    }
}

fn render_adornment(adornment: Adornment, disabled: bool, icon_size: Rems, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let color = if disabled {
        theme.fg_disabled()
    } else {
        theme.fg_muted()
    };

    match adornment.0 {
        AdornmentKind::Icon(icon) => div()
            .flex()
            .flex_none()
            .items_center()
            .child(icon.size(icon_size).text_color(color))
            .into_any_element(),
        AdornmentKind::Text(text) => div()
            .flex_none()
            .whitespace_nowrap()
            .text_color(color)
            .child(text)
            .into_any_element(),
        AdornmentKind::Element(element) => div().flex_none().child(element).into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::input::tests::focused_input_window;
    use gpui::TestAppContext;

    /// Draw one field, focused, in its own window, and type into it. Reports
    /// what the state holds afterwards — the question is whether a keystroke
    /// lands, not what the field looks like.
    fn type_into(
        cx: &mut TestAppContext,
        build: impl Fn(&Entity<InputState>, &App) -> TextField + 'static,
        content: &str,
        keystrokes: &str,
    ) -> String {
        let state = cx.update(|cx| cx.new(InputState::new_singleline));
        let content = content.to_string();
        state.update(cx, |state, cx| state.set_content(content, cx));

        let for_render = state.clone();
        let cx = focused_input_window(cx, &state, move |_window, cx| {
            build(&for_render, cx).into_any_element()
        });

        cx.run_until_parked();
        cx.simulate_keystrokes(keystrokes);
        cx.run_until_parked();

        state.read_with(cx, |state, _| state.content().to_string())
    }

    #[gpui::test]
    fn a_disabled_field_does_not_take_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(
            cx,
            |state, cx| text_field(state, cx).disabled(true),
            "kept",
            "x",
        );

        assert_eq!(after, "kept");
    }

    /// Without this, `InputState::set_read_only` on a field would produce a
    /// control that refuses edits while looking exactly like one that takes
    /// them.
    #[gpui::test]
    fn a_read_only_field_does_not_take_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(
            cx,
            |state, cx| text_field(state, cx).read_only(true),
            "kept",
            "x",
        );

        assert_eq!(after, "kept");
    }

    /// The check that the harness bites.
    #[gpui::test]
    fn an_editable_field_takes_a_keystroke(cx: &mut TestAppContext) {
        let after = type_into(cx, text_field, "kept", "x");

        assert_eq!(after, "xkept");
    }

    #[gpui::test]
    fn each_field_renders_under_its_own_state(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let (one, two) = cx.update(|cx| {
            (
                cx.new(InputState::new_singleline),
                cx.new(InputState::new_singleline),
            )
        });

        let (first, second, overridden) = cx.update(|cx| {
            (
                text_field(&one, cx).element_id(),
                text_field(&two, cx).element_id(),
                text_field(&one, cx).id("shared-state-left").element_id(),
            )
        });

        assert_eq!(first, text_field_element_id(one.entity_id()));
        assert_ne!(first, second);
        assert_eq!(overridden, ElementId::Name("shared-state-left".into()));
    }

    /// Focusing the field and focusing the text it wraps have to be the same
    /// act, or clicking the box would move focus away from the caret.
    #[gpui::test]
    fn the_fields_focus_handle_is_the_states(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let state = cx.update(|cx| cx.new(InputState::new_singleline));

        cx.update(|cx| {
            assert_eq!(
                text_field(&state, cx).focus_handle(cx),
                state.focus_handle(cx)
            );
        });
    }

    /// The disabled display falls back from content to placeholder to nothing.
    /// A disabled field renders static text rather than a live `Input`, so
    /// this is the whole of what it shows.
    #[test]
    fn a_disabled_field_shows_its_value_then_its_placeholder() {
        let placeholder = SharedString::from("Search");

        assert_eq!(
            disabled_display("typed", Some(&placeholder)),
            (SharedString::from("typed"), false)
        );
        assert_eq!(
            disabled_display("", Some(&placeholder)),
            (placeholder.clone(), true)
        );
        assert_eq!(disabled_display("", None), (SharedString::default(), false));
    }
}
