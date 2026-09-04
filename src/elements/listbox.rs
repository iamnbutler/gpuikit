//! Listbox — the popup two choosers are made of.
//!
//! This module is `pub(crate)` and deliberately not `pub`. It is not a
//! component: it is the thing [`crate::elements::select`] and
//! [`crate::elements::combobox`] are both built out of, which is exactly the
//! lift #154 prescribed for the moment a second caller appeared — *"lift
//! `Listbox` into a `pub(crate)` module named by both callers, do not make it
//! `pub` where it sits"*. Being `pub(crate)` also keeps it out of
//! `showcase_coverage`'s table, which keys off the literal string `pub mod `. That is the right answer rather
//! than a convenient one: a showcase page for "the inside of a select" would
//! be a page for something nobody can build.
//!
//! Everything here arrived from `src/elements/select.rs` unchanged in
//! behaviour. The one addition is [`ListboxFocus`], which is the whole
//! difference between the two callers:
//!
//! * A [`Select`](crate::elements::select::SelectState) **moves focus into the
//!   popup**. Its trigger is not a text field, there is nothing to type into,
//!   and the popup owning focus is what lets a plain `LISTBOX_CONTEXT` binding
//!   outrank an enclosing `Dialog`'s Escape.
//! * A [`Combobox`](crate::elements::combobox::ComboboxState) **must not**. The
//!   user is typing; taking focus would stop them. Its popup is a sibling of a
//!   focused `TextField`, it declares no key context, and its rows make no
//!   `active_descendant` claim — see [`crate::a11y::A11y::active_descendant`],
//!   which states that gpui honours that property only under a focused
//!   *ancestor* and that this arrangement "cannot be expressed".
//!
//! Both halves of the focus behaviour are conditional on that flag, not just
//! the `window.focus` call: a `restore_focus` read from a window whose focus is
//! about to stay where it is would hand focus *back* to the text field on
//! dismiss, which is right by accident today and wrong the day a combobox is
//! dismissed by a click somewhere else.

use crate::a11y::{A11y, Announce, FocusNext, FocusPrevious};
use crate::element_id::for_entity;
use crate::icons::Icons;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use gpui::{
    App, Context, DismissEvent, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyBinding, KeyDownEvent, ParentElement, Rems, Render, Role, ScrollHandle,
    SharedString, Styled, Window, actions, div, prelude::*, px,
};
use std::rc::Rc;

/// Which of the two focus arrangements a popup is in.
///
/// Not a `bool` parameter: `Listbox::build(…, true, …)` at a call site says
/// nothing about *what* is true, and the two callers differ on this one axis
/// in three separate places (whether focus moves, whether a restore handle is
/// kept, and whether a row may claim `active_descendant`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListboxFocus {
    /// The popup takes real keyboard focus and gives it back when it closes.
    /// A `Select`.
    Popup,
    /// Focus stays where the caller had it — on a text field the user is
    /// typing into. A `Combobox`.
    Caller,
}

/// The highlight-wrap arithmetic, shared with [`crate::elements::command`] and
/// [`crate::elements::context_menu`]. Lives once, in [`crate::selection`]; this
/// re-export keeps the name the listbox and command call sites already use.
pub(crate) use crate::selection::wrap_index as wrapped_index;

/// The default match: a case-insensitive substring test.
///
/// Shared by [`crate::elements::combobox`] and the showcase's command palette,
/// and replaceable in one line by either. The crate deliberately does not own a
/// matching *algorithm* — see `docs/issues/command.md` — but it does owe a
/// caller who has no opinion the answer nine in ten of them want.
pub(crate) fn matches_query(query: &str, haystack: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&query.to_lowercase())
}

actions!(
    select,
    [
        /// Move the listbox highlight to the next option, wrapping at the end.
        HighlightNext,
        /// Move the listbox highlight to the previous option, wrapping at the
        /// start.
        HighlightPrevious,
        /// Highlight the listbox's first option.
        HighlightFirst,
        /// Highlight the listbox's last option.
        HighlightLast,
        /// Choose the highlighted option and close the listbox.
        ChooseHighlighted,
        /// Close the listbox without choosing anything.
        DismissListbox,
    ]
);

/// The key context the listbox popup declares, and the one
/// [`bind_select_keys`] scopes its bindings to.
///
/// Public because the bindings are: an app assembling its own keymap needs both
/// halves. It is deeper than `Dialog`, which is what lets a select inside a
/// dialog keep Escape for itself.
pub(crate) const LISTBOX_CONTEXT: &str = "Listbox";

/// Bind the listbox popup's keys — arrows, Home / End, Enter, Space, Escape.
///
/// [`crate::init`] calls this, so an app that calls `gpuikit::init` gets the
/// keyboard model for free and nothing else is required. An app that assembles
/// its own keymap has to call it; without it the popup opens, takes focus and
/// answers nothing, which is the state this module's `# The keyboard` section
/// exists to have fixed.
///
/// Every binding is scoped to [`LISTBOX_CONTEXT`], so none of them is reachable
/// while the popup is closed. Tab is deliberately absent — see the module docs.
///
/// Space chooses as well as Enter, which is symmetric with Space opening the
/// popup from the trigger. The cost is that a type-ahead cannot contain a
/// space, which is a fair trade for one character of look-ahead.
pub fn bind_listbox_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", HighlightNext, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("up", HighlightPrevious, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("home", HighlightFirst, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("end", HighlightLast, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("enter", ChooseHighlighted, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("space", ChooseHighlighted, Some(LISTBOX_CONTEXT)),
        KeyBinding::new("escape", DismissListbox, Some(LISTBOX_CONTEXT)),
    ]);
}

/// The gap between the trigger and the listbox that drops out of it.
///
/// Applied through `anchored().offset(…)` and never as a margin on the
/// anchored child: `Anchored::prepaint` fits the *union of its children's
/// layout bounds* to the window, and a margin sits outside that union, so a
/// popup at the bottom of the window would be clamped correctly and then
/// pushed straight back out by its own margin. See `docs/overlays.md`.
pub(crate) const LISTBOX_GAP: Rems = Rems(0.25);

/// The popup that lists the options.
///
/// Private on purpose — see this module's docs and
/// `docs/menus-and-listboxes.md`. It takes plain labels rather than a row type
/// of its own: a listbox row is a label and whether it is the chosen one, and
/// anything richer belongs to the element that grew a need for it.
pub(crate) struct Listbox {
    /// The accessible name of the control this dropped out of. A popup is
    /// named after its trigger, not after itself.
    pub(crate) label: SharedString,
    pub(crate) options: Vec<SharedString>,
    /// The row that carries the selection, if any. `None` is a real state —
    /// a `Select` with no value marks nothing — rather than an index no option
    /// happens to have.
    pub(crate) selected_index: Option<usize>,
    /// The row the keyboard has got to, which is **not** the chosen one.
    ///
    /// It starts on the choice (or the first row, when there is no choice) and
    /// separates from it the moment an arrow key is pressed. `None` only for an
    /// empty list — a highlight with no row to be on.
    pub(crate) highlighted: Option<usize>,
    /// The rung of the trigger that opened this listbox, so a popup's rows are
    /// the same size as the control they dropped out of.
    pub(crate) size: ControlSize,
    /// Which of the two arrangements this popup is in — see [`ListboxFocus`].
    pub(crate) focus: ListboxFocus,
    pub(crate) focus_handle: FocusHandle,
    /// So a highlight the keyboard moved off-screen comes back into view. The
    /// popup scrolls; the highlight is the only thing that moves without a
    /// pointer to move it.
    pub(crate) scroll_handle: ScrollHandle,
    /// Whatever held focus when this popup opened — the trigger, in every path
    /// that exists today. Read at open time rather than passed in, because the
    /// trigger's handle is minted by `A11y::focusable` inside gpui's element
    /// state and `SelectState` never sees it.
    pub(crate) restore_focus: Option<FocusHandle>,
    pub(crate) on_select: Option<Rc<dyn Fn(usize, &mut Window, &mut App)>>,
}

impl EventEmitter<DismissEvent> for Listbox {}

impl Focusable for Listbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Listbox {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        label: SharedString,
        options: Vec<SharedString>,
        selected_index: Option<usize>,
        size: ControlSize,
        focus: ListboxFocus,
        on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            // Both halves are conditional, not just the `window.focus`. A
            // stored `restore_focus` a caller never asks for is harmless; a
            // popup that steals focus from the text field a `Combobox` user is
            // typing into is not.
            let restore_focus = match focus {
                // Read *before* the popup takes focus, or the handle read back
                // is the popup's own.
                ListboxFocus::Popup => window.focused(cx),
                ListboxFocus::Caller => None,
            };
            let focus_handle = cx.focus_handle();
            if focus == ListboxFocus::Popup {
                window.focus(&focus_handle, cx);
            }
            // The keyboard starts where the value is. With no value it starts
            // at the top, which is what makes the very first Down key press
            // land on the second row rather than nowhere.
            let highlighted = selected_index.or(if options.is_empty() { None } else { Some(0) });
            Self {
                label,
                options,
                selected_index,
                highlighted,
                size,
                focus,
                focus_handle,
                scroll_handle: ScrollHandle::new(),
                restore_focus,
                on_select: Some(Rc::new(on_select)),
            }
        })
    }

    /// Replace the rows under an open popup.
    ///
    /// A `Select`'s rows do not change while its popup is open; a `Combobox`'s
    /// change on every keystroke, which is the whole point of it. The highlight
    /// is reset to the first row rather than preserved by index: after a filter
    /// runs, row 3 is a different option than it was, so "keep the highlight
    /// where it was" would move it somewhere the user did not put it.
    pub(crate) fn set_options(
        &mut self,
        options: Vec<SharedString>,
        selected_index: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.options = options;
        self.selected_index = selected_index;
        self.highlighted = if self.options.is_empty() {
            None
        } else {
            Some(0)
        };
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    pub(crate) fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_select) = &self.on_select {
            let on_select = on_select.clone();
            on_select(index, window, cx);
        }
        // Choosing a row is something the user did *to this control*, so focus
        // belongs back on the control — whether the row was clicked or the
        // keyboard committed it.
        self.dismiss(true, window, cx);
    }

    /// Close the popup.
    ///
    /// `restore_focus` is what separates the keyboard's closes from a click
    /// outside. A keyboard close has to hand focus back, or the user is left
    /// with focus on an element that no longer exists. A click outside must
    /// **not**: that click is about to focus whatever it landed on, and pulling
    /// focus to the trigger first would fight it.
    pub(crate) fn dismiss(
        &mut self,
        restore_focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if restore_focus {
            if let Some(handle) = self.restore_focus.clone() {
                window.focus(&handle, cx);
            }
        }
        cx.emit(DismissEvent);
    }

    /// The single place the highlight moves, so `scroll_to_item` cannot be
    /// forgotten on one path and remembered on the others.
    pub(crate) fn highlight(&mut self, index: usize, cx: &mut Context<Self>) {
        self.highlighted = Some(index);
        self.scroll_handle.scroll_to_item(index);
        cx.notify();
    }

    /// Move the highlight by `delta` rows, wrapping at both ends.
    ///
    /// Wrapping is `rem_euclid` rather than a pair of bounds checks because the
    /// two ends are the same case, and a signed remainder would send `-1` off
    /// the front of the list.
    pub(crate) fn move_highlight(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some(next) = wrapped_index(self.highlighted, delta, self.options.len()) {
            self.highlight(next, cx);
        }
    }

    /// Home and End: `last` picks which end.
    pub(crate) fn highlight_edge(&mut self, last: bool, cx: &mut Context<Self>) {
        let count = self.options.len();
        if count == 0 {
            return;
        }
        self.highlight(if last { count - 1 } else { 0 }, cx);
    }

    /// Commit the highlight to the value.
    ///
    /// Nothing highlighted means an empty list, and closing on a key that chose
    /// nothing would be a worse answer than doing nothing at all.
    pub(crate) fn choose_highlighted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.highlighted {
            self.select(index, window, cx);
        }
    }

    /// Highlight the next option whose label starts with `character`.
    ///
    /// The search starts *after* the current highlight and wraps, so pressing
    /// the same letter again walks the options that share it. One character, no
    /// buffer, no timer — a buffer would need a timeout to expire, and a
    /// timeout is a behaviour to get wrong for an affordance this small.
    pub(crate) fn type_ahead(&mut self, character: char, cx: &mut Context<Self>) {
        let count = self.options.len();
        if count == 0 {
            return;
        }
        let start = self.highlighted.map_or(0, |current| current + 1);

        for offset in 0..count {
            let index = (start + offset) % count;
            let starts_with = self.options[index]
                .chars()
                .next()
                .is_some_and(|first| first.to_lowercase().eq(character.to_lowercase()));
            if starts_with {
                self.highlight(index, cx);
                return;
            }
        }
    }

    /// What row `index` announces.
    ///
    /// A method rather than a call inlined into `render` so that a test can
    /// read the same value the element reports. It has to: the active
    /// descendant is applied at paint time behind gpui's `a11y.is_active()`,
    /// which no test here can switch on, so the declaration is the only thing
    /// there is to hold. See [`A11y::active_descendant`].
    pub(crate) fn row_a11y(&self, index: usize) -> A11y {
        option_a11y(
            self.options[index].clone(),
            self.selected_index == Some(index),
            self.highlighted == Some(index),
            self.focus == ListboxFocus::Popup,
            index,
            self.options.len(),
        )
    }

    /// The character a key press typed, if it typed one.
    ///
    /// `Keystroke::with_simulated_ime` fills a `key_char` in for `space`, `tab`
    /// and `enter` (`" "`, `"\t"`, `"\n"`), so trusting `key_char` alone would
    /// make Tab a type-ahead for options starting with a tab. Whitespace,
    /// control characters and anything held with a modifier are rejected here
    /// rather than searched for.
    pub(crate) fn typed_character(event: &KeyDownEvent) -> Option<char> {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return None;
        }

        let mut characters = event.keystroke.key_char.as_ref()?.chars();
        let character = characters.next()?;
        if characters.next().is_some() || character.is_whitespace() || character.is_control() {
            return None;
        }
        Some(character)
    }
}

/// The popup is named after the control it dropped out of, which is the only
/// name it has: its own contents are the options, not a label.
///
/// [`Role::ListBox`] is deliberately *not* in
/// [`crate::a11y::role_requires_a_name`] — naming it here is this element's
/// decision, not one binding `list.rs` and every future chooser.
impl Accessible for Listbox {
    fn a11y(&self) -> A11y {
        A11y::new(Role::ListBox).name(self.label.clone())
    }
}

/// What one row of the popup announces.
///
/// A free function rather than an [`Accessible`] impl because a row is a `div`
/// built inside a closure, not a component — and because this way a test can
/// read it without laying the popup out.
///
/// `is_selected` and `is_highlighted` are two different things and are both
/// here: the first is the control's value, the second is where the keyboard
/// has got to. Exactly one row is ever the active descendant, which is what
/// keeps gpui's two-claims-in-one-frame `debug_assert!` unreachable.
pub(crate) fn option_a11y(
    label: SharedString,
    is_selected: bool,
    is_highlighted: bool,
    may_claim_active_descendant: bool,
    index: usize,
    count: usize,
) -> A11y {
    A11y::new(Role::ListBoxOption)
        .name(label)
        .selected(is_selected)
        .active_descendant(is_highlighted && may_claim_active_descendant)
        // Both, together: a position with no size announces "3" out of nowhere.
        .position_in_set(index + 1)
        .size_of_set(count)
}

impl Render for Listbox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        div()
            // Unique only because a `Listbox` is always rendered as an
            // `Entity<_>`, which puts an `ElementId::View` above it.
            .id(for_entity("gpuikit-listbox", cx.entity_id()))
            .announce(self.a11y())
            // Focus stays declared here rather than through `A11y`, and this is
            // a standing reason rather than a stopgap: `A11y::focus_handle`
            // applies `.tab_stop(true)`, and a transient overlay that already
            // holds focus must not also be in the tab order. `A11y` offers only
            // "tab stop" or "no focus"; a third answer — takes focus, is not a
            // tab stop — would let this go through the convention like
            // everything else.
            // Only when this popup is the focused thing. A `Combobox`'s
            // popup is a sibling of a text field that keeps real focus, so
            // neither the focus handle nor `LISTBOX_CONTEXT` would ever be on
            // the dispatch path — and a key context that is never reached is
            // better not declared than declared and misleading.
            .when(self.focus == ListboxFocus::Popup, |this| {
                this.track_focus(&focus_handle).key_context(LISTBOX_CONTEXT)
            })
            .on_action(cx.listener(|this, _: &HighlightNext, _window, cx| {
                this.move_highlight(1, cx);
            }))
            .on_action(cx.listener(|this, _: &HighlightPrevious, _window, cx| {
                this.move_highlight(-1, cx);
            }))
            .on_action(cx.listener(|this, _: &HighlightFirst, _window, cx| {
                this.highlight_edge(false, cx);
            }))
            .on_action(cx.listener(|this, _: &HighlightLast, _window, cx| {
                this.highlight_edge(true, cx);
            }))
            .on_action(cx.listener(|this, _: &ChooseHighlighted, window, cx| {
                this.choose_highlighted(window, cx);
            }))
            .on_action(cx.listener(|this, _: &DismissListbox, window, cx| {
                this.dismiss(true, window, cx);
            }))
            // Tab is not a binding of ours — `a11y`'s is context-less, and a
            // context-less binding outranks every scoped one. It is an action
            // listener, and a bubble-phase listener on the focused element runs
            // before any ancestor's `moves_focus_on_tab`. Focus goes back to
            // the trigger first so that "the next tab stop" is the one after
            // the control, not after a popup that has just stopped existing.
            .on_action(cx.listener(|this, _: &FocusNext, window, cx| {
                this.dismiss(true, window, cx);
                window.focus_next(cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPrevious, window, cx| {
                this.dismiss(true, window, cx);
                window.focus_prev(cx);
            }))
            // The only key that is not an action: a binding per letter is not a
            // keymap. gpui runs key-down listeners only when no binding
            // matched, which is what keeps this out of the other keys' way.
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if let Some(character) = Self::typed_character(event) {
                    this.type_ahead(character, cx);
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(false, window, cx);
            }))
            .min_w(px(120.))
            .max_h(px(480.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .on_scroll_wheel(|_, _, cx| {
                cx.stop_propagation();
            })
            .bg(theme.surface())
            .border_1()
            .border_color(theme.border())
            .rounded(metrics.radius)
            .shadow_lg()
            .py(metrics.padding_y())
            .flex()
            .flex_col()
            .children(self.options.iter().enumerate().map(|(index, label)| {
                let is_selected = self.selected_index == Some(index);
                let is_highlighted = self.highlighted == Some(index);
                let label = label.clone();
                let theme = cx.theme();
                let a11y = self.row_a11y(index);

                // The chosen row is a check and the highlighted row is a fill,
                // because they are two states and one accent fill cannot say
                // both. The check sits in a slot every row reserves — the shape
                // `context_menu.rs` already uses — so labels stay aligned
                // whether or not their row is the chosen one.
                let (fg, check_color) = if is_highlighted {
                    (theme.bg(), theme.bg())
                } else {
                    (theme.fg(), theme.accent())
                };

                let row =
                    div()
                        .id(ElementId::NamedInteger(
                            "listbox-option".into(),
                            index as u64,
                        ))
                        .announce(a11y)
                        .flex()
                        .items_center()
                        .gap(metrics.gap)
                        .h(metrics.height)
                        .px(metrics.padding_x * 1.5)
                        .text_size(metrics.text_size)
                        .line_height(metrics.line_height)
                        .cursor_pointer()
                        .text_color(fg)
                        .when(is_highlighted, |this| this.bg(theme.accent()))
                        // Hovering moves the highlight rather than drawing a second
                        // one: a pointer and a keyboard fighting over which row is
                        // "current" is exactly the two-affordances-one-state
                        // problem this rendering exists to end.
                        .on_hover(cx.listener(move |this, hovered: &bool, _window, cx| {
                            if *hovered {
                                this.highlight(index, cx);
                            }
                        }))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.select(index, window, cx);
                        }))
                        .child(div().w(metrics.text_size).flex_shrink_0().when(
                            is_selected,
                            |this| {
                                this.child(
                                    Icons::check()
                                        .size(metrics.text_size)
                                        .text_color(check_color),
                                )
                            },
                        ))
                        .child(label);

                // Lets a test press a key and then read which row the highlight
                // landed on from where it was actually drawn. A no-op outside a
                // test build.
                #[cfg(test)]
                let row = row.debug_selector(move || format!("gpuikit-select-option-{index}"));

                row
            }))
    }
}
