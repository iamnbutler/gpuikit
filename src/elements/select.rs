//! Select
//!
//! **A select is a listbox: it offers *values* to choose between, and the
//! choice persists. A menu — `src/elements/context_menu.rs` — offers *actions*
//! to invoke, and nothing stays selected once one has been.** That sentence is
//! the whole of the boundary between the two families, and it is why they
//! share only `docs/overlays.md` and nothing else. Read
//! `docs/menus-and-listboxes.md` before adding a third component to this
//! neighbourhood.
//!
//! This is the crate's only listbox. `Dropdown` was the same component under a
//! second name — the same bordered trigger, the same chevron, the same popup
//! one gap below it — and was deleted in favour of this one;
//! `dropdown(id, options, value)` is now `select(id, options).selected(value)`.
//!
//! The popup, `Listbox`, is private to this module on purpose. A chooser that
//! wants a listbox is a chooser this file should grow, not a second element
//! built on this one's internals — which is precisely the state that made
//! `Select` and `Dropdown` indistinguishable in the first place.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::traits::disableable::Disableable;
//!
//! // Create a select with enum options
//! #[derive(Clone, PartialEq)]
//! enum Country { US, UK, CA }
//!
//! let select_state = cx.new(|_cx| {
//!     SelectState::new(
//!         select(
//!             "country-select",
//!             "Country",
//!             vec![
//!                 (Country::US, "United States"),
//!                 (Country::UK, "United Kingdom"),
//!                 (Country::CA, "Canada"),
//!             ],
//!         )
//!         .selected(Country::US) // Optional default value
//!         .placeholder("Choose a country...")
//!         .on_change(|value, _window, _cx| {
//!             println!("Selected: {:?}", value);
//!         })
//!         .disabled(false)
//!     )
//! });
//! ```
//!
//! # Accessibility
//!
//! Three nodes, following `src/a11y.rs`. The trigger announces
//! [`Role::ComboBox`] with the control's name, whether it is `expanded`, and —
//! as its **value** — the label of the chosen option. The popup announces
//! [`Role::ListBox`], named after the control it dropped out of. Each row
//! announces [`Role::ListBoxOption`] with `selected`, its position in the set
//! and the size of the set.
//!
//! **The name is a constructor argument** because none of the naming sources
//! `a11y`'s section 2 allows was available. A select's visible text is its
//! *value*, so naming the control after it would rename the control every time
//! the user changed it; the placeholder disappears the moment a choice is made
//! and defaults to "Select…"; and gpui has no `labelled_by` builder, so a
//! `Field` or `Label` beside the control cannot name it either. `ComboBox` is
//! in [`crate::a11y::role_requires_a_name`], so an honest role forces the
//! argument.
//!
//! The popup owns real keyboard focus while it is open, and the **highlighted**
//! row — not the chosen one — claims `aria-activedescendant` through
//! [`A11y::active_descendant`]. gpui spells that property on the descendant and
//! honours it only under a focused *ancestor*, so the APG arrangement where
//! focus stays on the trigger and points across at the popup is not available
//! here: the popup is the trigger's sibling under a `div().relative()`, not its
//! child. Focus on the popup, claim on the row, is the arrangement gpui can
//! express — and the one this element already had half of.
//!
//! # The keyboard
//!
//! Two states, not one. The **choice** is the control's value and persists; the
//! **highlight** is where the keyboard has got to and dies with the popup. They
//! start on the same row — the chosen one, or the first row when nothing is
//! chosen — and separate the moment an arrow key is pressed. The chosen row is
//! drawn with a check, the highlighted row with a fill, because two states with
//! one affordance between them is the bug this section exists to avoid.
//!
//! | Key | What it does |
//! | --- | --- |
//! | Down / Up | Move the highlight one row, wrapping at each end |
//! | Home / End | Highlight the first / last row |
//! | Enter, Space | Choose the highlighted row and close |
//! | Escape | Close, choosing nothing |
//! | Tab, Shift-Tab | Close, then move focus on |
//! | a printable character | Highlight the next row whose label starts with it |
//!
//! Every close the keyboard asked for gives the trigger its focus back. A click
//! outside deliberately does **not**: that click is on its way to focusing
//! whatever it landed on, and pulling focus to the trigger first would fight it.
//!
//! **The keys are actions, not an `on_key_down`.** gpui dispatches bound actions
//! before key-down listeners, so a raw Escape handler here would never see the
//! keystroke whenever a select sits inside a `Dialog` — `dialog`'s `escape` →
//! `Close` binding would take it first and close the dialog out from under the
//! open popup. A binding in the deeper [`LISTBOX_CONTEXT`] wins that contest
//! instead. (`context_menu.rs` still uses a raw `on_key_down` and still has that
//! defect; it is a different element and its own change.) The bindings live in
//! [`bind_select_keys`], which [`crate::init`] calls — an app that assembles its
//! own keymap has to call it, which is why it is public.
//!
//! **Tab is the exception, and is not a binding.** `a11y`'s `tab` → `FocusNext`
//! binding carries no key context, and a context-less binding counts as the
//! deepest there is, so no `Listbox`-scoped binding could outrank it. The popup
//! answers `FocusNext` / `FocusPrevious` with an `on_action` listener instead:
//! bubble-phase action listeners run deepest-first, so the popup gets there
//! before any ancestor's `moves_focus_on_tab`. Without it, Tab moves focus away
//! and leaves the popup open behind it.
//!
//! **Type-ahead is the exception in the other direction**, and is the one key
//! that stays an `on_key_down`: a binding per letter is not a keymap. It only
//! runs when no binding matched, which is exactly what keeps it out of the other
//! keys' way. One character, no buffer and no timer — the search starts *after*
//! the current highlight and wraps, so pressing the same letter again walks the
//! options that share it.

use crate::a11y::{A11y, Announce, FocusNext, FocusPrevious};
use crate::element_id::for_entity;
use crate::theme::{focus_ring, ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    actions, anchored, deferred, div, point, prelude::*, px, App, Context, DismissEvent, ElementId,
    Entity, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, KeyDownEvent,
    ParentElement, Rems, Render, Role, ScrollHandle, SharedString, Styled, Window,
};

use crate::icons::Icons;
use std::rc::Rc;

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
pub const LISTBOX_CONTEXT: &str = "Listbox";

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
pub fn bind_select_keys(cx: &mut App) {
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

/// The width a trigger will not shrink below, so a short label still gives the
/// chevron somewhere to sit.
const MIN_TRIGGER_WIDTH: Rems = Rems(6.25);

/// The gap between the trigger and the listbox that drops out of it.
///
/// Applied through `anchored().offset(…)` and never as a margin on the
/// anchored child: `Anchored::prepaint` fits the *union of its children's
/// layout bounds* to the window, and a margin sits outside that union, so a
/// popup at the bottom of the window would be clamped correctly and then
/// pushed straight back out by its own margin. See `docs/overlays.md`.
const LISTBOX_GAP: Rems = Rems(0.25);

/// Event emitted when the select value changes.
pub struct SelectChanged;

/// The popup that lists the options.
///
/// Private on purpose — see this module's docs and
/// `docs/menus-and-listboxes.md`. It takes plain labels rather than a row type
/// of its own: a listbox row is a label and whether it is the chosen one, and
/// anything richer belongs to the element that grew a need for it.
struct Listbox {
    /// The accessible name of the control this dropped out of. A popup is
    /// named after its trigger, not after itself.
    label: SharedString,
    options: Vec<SharedString>,
    /// The row that carries the selection, if any. `None` is a real state —
    /// a `Select` with no value marks nothing — rather than an index no option
    /// happens to have.
    selected_index: Option<usize>,
    /// The row the keyboard has got to, which is **not** the chosen one.
    ///
    /// It starts on the choice (or the first row, when there is no choice) and
    /// separates from it the moment an arrow key is pressed. `None` only for an
    /// empty list — a highlight with no row to be on.
    highlighted: Option<usize>,
    /// The rung of the trigger that opened this listbox, so a popup's rows are
    /// the same size as the control they dropped out of.
    size: ControlSize,
    focus_handle: FocusHandle,
    /// So a highlight the keyboard moved off-screen comes back into view. The
    /// popup scrolls; the highlight is the only thing that moves without a
    /// pointer to move it.
    scroll_handle: ScrollHandle,
    /// Whatever held focus when this popup opened — the trigger, in every path
    /// that exists today. Read at open time rather than passed in, because the
    /// trigger's handle is minted by `A11y::focusable` inside gpui's element
    /// state and `SelectState` never sees it.
    restore_focus: Option<FocusHandle>,
    on_select: Option<Rc<dyn Fn(usize, &mut Window, &mut App)>>,
}

impl EventEmitter<DismissEvent> for Listbox {}

impl Focusable for Listbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Listbox {
    #[allow(clippy::too_many_arguments)]
    fn build(
        label: SharedString,
        options: Vec<SharedString>,
        selected_index: Option<usize>,
        size: ControlSize,
        on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            // Read *before* the popup takes focus, or the handle read back is
            // the popup's own.
            let restore_focus = window.focused(cx);
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
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
                focus_handle,
                scroll_handle: ScrollHandle::new(),
                restore_focus,
                on_select: Some(Rc::new(on_select)),
            }
        })
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
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
    fn dismiss(&mut self, restore_focus: bool, window: &mut Window, cx: &mut Context<Self>) {
        if restore_focus {
            if let Some(handle) = self.restore_focus.clone() {
                window.focus(&handle, cx);
            }
        }
        cx.emit(DismissEvent);
    }

    /// The single place the highlight moves, so `scroll_to_item` cannot be
    /// forgotten on one path and remembered on the others.
    fn highlight(&mut self, index: usize, cx: &mut Context<Self>) {
        self.highlighted = Some(index);
        self.scroll_handle.scroll_to_item(index);
        cx.notify();
    }

    /// Move the highlight by `delta` rows, wrapping at both ends.
    ///
    /// Wrapping is `rem_euclid` rather than a pair of bounds checks because the
    /// two ends are the same case, and a signed remainder would send `-1` off
    /// the front of the list.
    fn move_highlight(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.options.len();
        if count == 0 {
            return;
        }
        let next = match self.highlighted {
            Some(current) => (current as isize + delta).rem_euclid(count as isize) as usize,
            // Only reachable if a list went from empty to full while open,
            // which it cannot today. Entering from the near end is still the
            // answer that reads right.
            None if delta < 0 => count - 1,
            None => 0,
        };
        self.highlight(next, cx);
    }

    /// Home and End: `last` picks which end.
    fn highlight_edge(&mut self, last: bool, cx: &mut Context<Self>) {
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
    fn choose_highlighted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn type_ahead(&mut self, character: char, cx: &mut Context<Self>) {
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
    fn row_a11y(&self, index: usize) -> A11y {
        option_a11y(
            self.options[index].clone(),
            self.selected_index == Some(index),
            self.highlighted == Some(index),
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
    fn typed_character(event: &KeyDownEvent) -> Option<char> {
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
fn option_a11y(
    label: SharedString,
    is_selected: bool,
    is_highlighted: bool,
    index: usize,
    count: usize,
) -> A11y {
    A11y::new(Role::ListBoxOption)
        .name(label)
        .selected(is_selected)
        .active_descendant(is_highlighted)
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
            .id(for_entity("select-listbox", cx.entity_id()))
            .announce(self.a11y())
            // Focus stays declared here rather than through `A11y`, and this is
            // a standing reason rather than a stopgap: `A11y::focus_handle`
            // applies `.tab_stop(true)`, and a transient overlay that already
            // holds focus must not also be in the tab order. `A11y` offers only
            // "tab stop" or "no focus"; a third answer — takes focus, is not a
            // tab stop — would let this go through the convention like
            // everything else.
            .track_focus(&focus_handle)
            .key_context(LISTBOX_CONTEXT)
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
                            "select-option".into(),
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

/// Builder for creating a select component.
///
/// Use the [`select`] function to create an instance.
pub struct Select<T: Clone + PartialEq + 'static> {
    id: ElementId,
    label: SharedString,
    options: Vec<(T, SharedString)>,
    selected: Option<T>,
    placeholder: SharedString,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

/// Creates a new select builder.
///
/// # Arguments
///
/// * `id` - Unique identifier for the select
/// * `label` - The accessible name: what this control is *for*, as distinct
///   from what is currently chosen in it. Required rather than optional
///   because a select's visible text is its value and gpui has no
///   `labelled_by` builder — see this module's `# Accessibility` section
/// * `options` - Vector of (value, label) tuples
///
/// # Example
///
/// ```ignore
/// select(
///     "my-select",
///     "Option",
///     vec![("a", "Option A"), ("b", "Option B")],
/// )
/// .selected("a")
/// .placeholder("Choose an option...")
/// ```
pub fn select<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    options: Vec<(T, impl Into<SharedString>)>,
) -> Select<T> {
    Select::new(id, label, options)
}

impl<T: Clone + PartialEq + 'static> Select<T> {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        options: Vec<(T, impl Into<SharedString>)>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: options
                .into_iter()
                .map(|(value, label)| (value, label.into()))
                .collect(),
            selected: None,
            placeholder: "Select...".into(),
            on_change: None,
            full_width: false,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    /// Set the initially selected value.
    ///
    /// A select with no value shows its placeholder and marks no row. This
    /// method is what the deleted `Dropdown`'s required third argument became.
    pub fn selected(mut self, value: T) -> Self {
        self.selected = Some(value);
        self
    }

    /// Set the placeholder text shown when no value is selected.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Register a callback for when the selection changes.
    pub fn on_change(mut self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Make the select expand to fill available width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

impl<T: Clone + PartialEq + 'static> Disableable for Select<T> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Clone + PartialEq + 'static> ControlSized for Select<T> {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

/// Stateful select component that manages the option popup.
///
/// Create using [`Select`] and wrap in an Entity:
///
/// ```ignore
/// let state = cx.new(|_cx| SelectState::new(select(...)));
/// ```
pub struct SelectState<T: Clone + PartialEq + 'static> {
    id: ElementId,
    label: SharedString,
    options: Vec<(T, SharedString)>,
    /// The currently selected value, if any.
    pub selected: Option<T>,
    placeholder: SharedString,
    listbox: Option<Entity<Listbox>>,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<SelectChanged> for SelectState<T> {}

impl<T: Clone + PartialEq + 'static> SelectState<T> {
    pub fn new(select: Select<T>) -> Self {
        Self {
            id: select.id,
            label: select.label,
            options: select.options,
            selected: select.selected,
            placeholder: select.placeholder,
            listbox: None,
            on_change: select.on_change,
            full_width: select.full_width,
            disabled: select.disabled,
            size: select.size,
        }
    }

    /// Get the label of the currently selected option, or the placeholder if none selected.
    fn display_label(&self) -> (SharedString, bool) {
        match &self.selected {
            Some(selected) => {
                let label = self
                    .options
                    .iter()
                    .find(|(v, _)| v == selected)
                    .map(|(_, label)| label.clone())
                    .unwrap_or_else(|| self.placeholder.clone());
                (label, false)
            }
            None => (self.placeholder.clone(), true),
        }
    }

    /// Get the index of the currently selected option, or `None` if nothing is
    /// selected.
    fn selected_index(&self) -> Option<usize> {
        self.selected
            .as_ref()
            .and_then(|selected| self.options.iter().position(|(v, _)| v == selected))
    }

    /// Update the selected value programmatically.
    pub fn set_selected(&mut self, value: Option<T>, cx: &mut Context<Self>) {
        self.selected = value;
        cx.emit(SelectChanged);
        cx.notify();
    }

    /// Check if the listbox is currently open.
    pub fn is_open(&self) -> bool {
        self.listbox.is_some()
    }

    /// Check if the select is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Set the disabled state programmatically.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        if disabled && self.listbox.is_some() {
            self.listbox = None;
        }
        cx.notify();
    }

    /// Clear the selection.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.selected = None;
        cx.emit(SelectChanged);
        cx.notify();
    }

    fn toggle_listbox(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }

        if self.listbox.is_some() {
            self.listbox = None;
            cx.notify();
            return;
        }

        let options: Vec<SharedString> = self
            .options
            .iter()
            .map(|(_, label)| label.clone())
            .collect();

        let selected_index = self.selected_index();
        let values: Vec<T> = self.options.iter().map(|(v, _)| v.clone()).collect();
        let on_change = self.on_change.clone();

        let entity = cx.entity().downgrade();
        let listbox = Listbox::build(
            self.label.clone(),
            options,
            selected_index,
            self.size,
            move |index, window, cx| {
                if let Some(value) = values.get(index).cloned() {
                    if let Some(on_change) = &on_change {
                        on_change(value.clone(), window, cx);
                    }
                    if let Some(entity) = entity.upgrade() {
                        entity.update(cx, |state, cx| {
                            state.selected = Some(value);
                            cx.emit(SelectChanged);
                            cx.notify();
                        });
                    }
                }
            },
            window,
            cx,
        );

        cx.subscribe_in(
            &listbox,
            window,
            |this, _, _event: &DismissEvent, _window, cx| {
                this.listbox = None;
                cx.notify();
            },
        )
        .detach();

        self.listbox = Some(listbox);
        cx.notify();
    }

    /// The label of the chosen option, or `None` when nothing is chosen.
    ///
    /// Not `display_label`: that falls back to the placeholder, and a
    /// placeholder is not a value. gpui has no `aria_placeholder` in
    /// [`A11y`] yet, so an empty select reports no value at all rather than
    /// reporting "Select…" as one.
    fn selected_label(&self) -> Option<SharedString> {
        self.selected_index()
            .and_then(|index| self.options.get(index))
            .map(|(_, label)| label.clone())
    }

    pub fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.listbox.is_some();
        // Before `theme` borrows `cx`, and before the label below shadows the
        // control's own name.
        let a11y = self.a11y();
        let (label, is_placeholder) = self.display_label();
        let full_width = self.full_width;
        let disabled = self.disabled;
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        let border_color = if disabled {
            theme.border_subtle()
        } else if is_open {
            theme.input_border_focused()
        } else {
            theme.input_border()
        };

        let text_color = if disabled {
            theme.fg_disabled()
        } else if is_placeholder {
            theme.input_placeholder()
        } else {
            theme.fg()
        };

        let trigger = div()
            .id(self.id.clone())
            .announce(a11y)
            .focus_visible(|style| style.shadow(focus_ring(theme.accent())))
            .flex()
            .items_center()
            .justify_between()
            // A declared height. The trigger used to be padding plus a
            // line box, which is why it could not line up with
            // anything.
            .h(metrics.height)
            .gap(metrics.gap)
            .px(metrics.padding_x)
            .min_w(MIN_TRIGGER_WIDTH)
            .when(full_width, |this| this.w_full())
            .bg(theme.input_bg())
            .border_1()
            .border_color(border_color)
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .line_height(metrics.line_height)
            .text_color(text_color)
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(|style| style.border_color(theme.input_border_hover()))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_listbox(window, cx);
                    }))
            })
            .child(label)
            .child(
                div().flex().items_center().justify_center().child(
                    Icons::chevron_down()
                        .size(metrics.text_size)
                        .text_color(theme.fg_muted()),
                ),
            );

        // Lets a test read where the trigger and its popup were actually laid
        // out, rather than hardcoding either one's metrics. A no-op outside a
        // test build — the same shape `context_menu.rs` uses.
        #[cfg(test)]
        let trigger = trigger.debug_selector(|| "gpuikit-select-trigger".into());

        // The gap goes on the anchored element, not on its child: gpui fits
        // the union of the child's *layout bounds* to the window, and a margin
        // is outside it. See `LISTBOX_GAP`.
        let gap = LISTBOX_GAP.to_pixels(window.rem_size());

        div()
            .relative()
            .when(full_width, |this| this.w_full())
            .child(trigger)
            .when_some(self.listbox.clone(), |this, listbox| {
                let popup = div().occlude().child(listbox);

                #[cfg(test)]
                let popup = popup.debug_selector(|| "gpuikit-select-popup".into());

                this.child(
                    deferred(anchored().offset(point(px(0.), gap)).child(popup)).with_priority(1),
                )
            })
    }
}

/// The trigger carries the state because it is the control the user operates
/// and the only one of the two nodes with an id of its own — the same rule
/// `SidebarTrigger` follows for `aria-expanded`.
///
/// The name is the constructor argument; the *value* is the chosen option's
/// label. Confusing the two is the mistake this element is most exposed to: a
/// select named after its own visible text would be renamed every time the
/// user changed it.
impl<T: Clone + PartialEq + 'static> Accessible for SelectState<T> {
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::ComboBox)
            .name(self.label.clone())
            .expanded(self.listbox.is_some());

        let a11y = match self.selected_label() {
            Some(label) => a11y.text_value(label),
            None => a11y,
        };

        // A combo box a screen reader announces and a keyboard cannot reach is
        // exactly the defect `crate::a11y`'s section 4 exists for, so this is
        // never `not_focusable` while the control is live. A disabled one
        // leaves the tab order for the same reason a disabled `Button` does.
        if self.disabled {
            a11y.not_focusable("a disabled select has nothing for a keyboard to choose between")
        } else {
            a11y.focusable()
        }
    }
}

impl<T: Clone + PartialEq + 'static> Render for SelectState<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render(window, cx)
    }
}

/// Two of these are the one property `docs/overlays.md` claims and nothing else
/// could check: that a popup placed with `anchored()` really does stay inside
/// the window, *and* really does keep its gap from the trigger. Those two
/// pulled against each other while the gap was a margin on the anchored child —
/// gpui fits the union of the child's layout bounds to the window, and a margin
/// is outside it, so the popup was clamped correctly and then pushed back out
/// by the gap. They arrived here with the merge of `Dropdown` into `Select`,
/// which deleted the file they were written in.
///
/// The third is what replaced the `usize::MAX`-means-nothing sentinel this
/// module used to pass into the popup.
///
/// The keyboard tests are a different shape and are grouped under their own
/// heading below: each one opens a real window and presses a real key, because
/// a binding registered in the wrong context and one registered correctly are
/// indistinguishable until a keystroke walks the dispatch tree.
#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        px, size, Bounds, KeyUpEvent, Keystroke, Pixels, PlatformInput, Render, TestAppContext,
        VisualTestContext,
    };
    use std::ops::Deref;

    struct TestView {
        select: Entity<SelectState<usize>>,
        /// Dead space above the trigger, so a test can put it wherever in the
        /// window it needs it.
        top: Pixels,
    }

    impl Render for TestView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .size_full()
                .child(div().h(self.top).flex_shrink_0())
                .child(self.select.clone())
        }
    }

    /// What one opened select looks like from the outside: the state, the
    /// context to read it through, and where the two boxes ended up.
    struct Opened {
        select: Entity<SelectState<usize>>,
        cx: &'static mut VisualTestContext,
        trigger: Bounds<Pixels>,
        popup: Bounds<Pixels>,
    }

    /// Open a select `top` pixels down a window of `window_size`.
    ///
    /// `options` is how many rows the listbox has, which is how tall it is —
    /// enough of them and it fits neither below the trigger nor above it, so
    /// gpui refuses to flip and falls back to clamping into the window. That
    /// is the path the margin bug lived on.
    fn open_select(
        cx: &mut TestAppContext,
        window_size: gpui::Size<Pixels>,
        top: Pixels,
        options: usize,
        selected: Option<usize>,
    ) -> Opened {
        let labels: Vec<SharedString> = (0..options)
            .map(|index| SharedString::from(format!("Option {index}")))
            .collect();
        open_labelled(cx, window_size, top, labels, selected)
    }

    /// The same, with the row labels spelled out — what type-ahead needs, since
    /// every option a numbered list generates starts with the same letter.
    fn open_labelled(
        cx: &mut TestAppContext,
        window_size: gpui::Size<Pixels>,
        top: Pixels,
        labels: Vec<SharedString>,
        selected: Option<usize>,
    ) -> Opened {
        // `crate::init`, not `theme::init`: the keyboard model is bindings, and
        // `bind_select_keys` is in `init`. A test that only initialised the
        // theme would open a popup that answers nothing and would be testing
        // the wiring rather than the element.
        cx.update(crate::init);

        let window = cx.open_window(window_size, move |_window, cx| {
            let select = cx.new(|_cx| {
                let options: Vec<(usize, SharedString)> =
                    labels.iter().cloned().enumerate().collect();
                let mut builder = super::select("test-select", "Test select", options);
                if let Some(selected) = selected {
                    builder = builder.selected(selected);
                }
                SelectState::new(builder)
            });
            TestView { select, top }
        });

        let select = window
            .read_with(cx, |view, _cx| view.select.clone())
            .expect("the window's root view is the test view");

        let cx = VisualTestContext::from_window(*window.deref(), cx).into_mut();
        cx.run_until_parked();

        let trigger = cx
            .debug_bounds("gpuikit-select-trigger")
            .expect("the trigger should have been laid out");
        cx.simulate_click(trigger.center(), gpui::Modifiers::default());
        cx.run_until_parked();

        let popup = cx
            .debug_bounds("gpuikit-select-popup")
            .expect("the popup should have been laid out");

        Opened {
            select,
            cx,
            trigger,
            popup,
        }
    }

    /// Press and release `key`.
    ///
    /// `simulate_keystrokes` sends only `KeyDown` — enough for a bound action,
    /// and *not* enough for gpui's Enter/Space activation of the trigger, which
    /// synthesises its click from a matched key up. Same helper
    /// `elements::button`'s tests carry, for the same reason.
    fn press(cx: &mut VisualTestContext, key: &str) {
        cx.simulate_keystrokes(key);
        let keystroke = Keystroke::parse(key).expect("a parseable keystroke");
        cx.update(|window, cx| {
            window.dispatch_event(PlatformInput::KeyUp(KeyUpEvent { keystroke }), cx);
        });
        cx.run_until_parked();
    }

    /// Where the keyboard has got to in the open popup.
    fn highlighted_row(
        select: &Entity<SelectState<usize>>,
        cx: &VisualTestContext,
    ) -> Option<usize> {
        select.read_with(cx, |state, cx| {
            state
                .listbox
                .as_ref()
                .expect("the popup should still be open")
                .read(cx)
                .highlighted
        })
    }

    /// Whether the popup is open at all.
    fn is_open(select: &Entity<SelectState<usize>>, cx: &VisualTestContext) -> bool {
        select.read_with(cx, |state, _| state.is_open())
    }

    /// The control's value — the choice, as distinct from the highlight.
    fn chosen(select: &Entity<SelectState<usize>>, cx: &VisualTestContext) -> Option<usize> {
        select.read_with(cx, |state, _| state.selected)
    }

    /// Where row `index` was actually drawn.
    ///
    /// `debug_bounds` keys are `&'static str` and a row's is built per index, so
    /// the test leaks one small string per lookup. That is the whole cost of not
    /// hardcoding the popup's row metrics here.
    fn row_bounds(cx: &mut VisualTestContext, index: usize) -> Bounds<Pixels> {
        let selector: &'static str =
            Box::leak(format!("gpuikit-select-option-{index}").into_boxed_str());
        cx.debug_bounds(selector)
            .unwrap_or_else(|| panic!("row {index} should have been laid out"))
    }

    // --- the keyboard ---
    //
    // Every one of these opens a real window and presses a real key. There is
    // no other way to test a keymap: a binding that is registered in the wrong
    // context, or an action nothing listens for, is indistinguishable from a
    // working one until a keystroke goes through the dispatch tree.

    /// The highlight starts on the choice, so Down from a fresh popup moves off
    /// the chosen row rather than from nowhere. With no choice it starts at the
    /// top — the first row, not "before the first row".
    #[gpui::test]
    fn the_highlight_starts_on_the_chosen_row_or_the_first_one(cx: &mut TestAppContext) {
        let window = size(px(320.), px(800.));

        let opened = open_select(cx, window, px(40.), 4, Some(2));
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(2),
            "an open popup starts the keyboard on the value the control holds"
        );

        let opened = open_select(cx, window, px(40.), 4, None);
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(0),
            "with nothing chosen there is still somewhere for the keyboard to be"
        );
    }

    /// The whole point of the two states: an arrow key moves the highlight and
    /// leaves the value alone. A listbox that chose as it moved would fire
    /// `on_change` for every row the user passed over.
    #[gpui::test]
    fn an_arrow_key_moves_the_highlight_and_chooses_nothing(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 4, Some(1));

        press(opened.cx, "down");
        assert_eq!(highlighted_row(&opened.select, opened.cx), Some(2));
        press(opened.cx, "up");
        press(opened.cx, "up");
        assert_eq!(highlighted_row(&opened.select, opened.cx), Some(0));

        assert!(is_open(&opened.select, opened.cx), "arrows do not close");
        assert_eq!(
            chosen(&opened.select, opened.cx),
            Some(1),
            "moving the highlight changed the control's value"
        );
    }

    /// Both ends, because they are one case in the code (`rem_euclid`) and two
    /// cases for the user.
    #[gpui::test]
    fn the_highlight_wraps_at_both_ends(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(0));

        press(opened.cx, "up");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(2),
            "up from the first row wraps to the last"
        );
        press(opened.cx, "down");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(0),
            "down from the last row wraps to the first"
        );
    }

    #[gpui::test]
    fn home_and_end_jump_to_the_ends(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 5, Some(2));

        press(opened.cx, "end");
        assert_eq!(highlighted_row(&opened.select, opened.cx), Some(4));
        press(opened.cx, "home");
        assert_eq!(highlighted_row(&opened.select, opened.cx), Some(0));
        assert!(is_open(&opened.select, opened.cx));
    }

    /// Enter is the commit: the highlight becomes the value, and the popup
    /// closes because there is nothing left to choose.
    #[gpui::test]
    fn enter_chooses_the_highlighted_row_and_closes(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 4, Some(0));

        press(opened.cx, "down");
        press(opened.cx, "down");
        press(opened.cx, "enter");

        assert_eq!(chosen(&opened.select, opened.cx), Some(2));
        assert!(
            !is_open(&opened.select, opened.cx),
            "Enter closes the popup"
        );
    }

    /// Escape closes and chooses nothing — and hands the trigger back its
    /// focus, which is what the second half of this test is for. There is no
    /// direct assertion available: by the time a test can call
    /// `window.focused()`, focus has already moved, and the trigger's handle is
    /// minted inside gpui's element state where nothing here can reach it. What
    /// *is* observable is that the trigger answers a keystroke afterwards, and
    /// only a focused trigger does.
    #[gpui::test]
    fn escape_closes_without_choosing_and_gives_the_trigger_its_focus_back(
        cx: &mut TestAppContext,
    ) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(1));

        press(opened.cx, "down");
        press(opened.cx, "escape");

        assert!(!is_open(&opened.select, opened.cx), "Escape closes");
        assert_eq!(
            chosen(&opened.select, opened.cx),
            Some(1),
            "Escape left the highlight where it was and the value alone"
        );

        press(opened.cx, "enter");
        assert!(
            is_open(&opened.select, opened.cx),
            "Enter reopened nothing, so Escape did not give the trigger its focus back"
        );
    }

    /// Tab is not one of this module's bindings — it is an `on_action` listener
    /// that beats the ancestor's by being deeper in the bubble. Without it the
    /// popup would still be open behind the focus that just left it.
    #[gpui::test]
    fn tab_closes_the_popup_on_its_way_past(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(1));

        press(opened.cx, "tab");

        assert!(
            !is_open(&opened.select, opened.cx),
            "Tab moved focus and left an orphaned popup behind it"
        );
        assert_eq!(
            chosen(&opened.select, opened.cx),
            Some(1),
            "Tab is not a way of choosing"
        );
    }

    /// One character, and the search starts *after* the highlight — so the same
    /// letter twice walks the options that share it rather than sticking on the
    /// first. That walk is the reason there is no buffer and no timer.
    #[gpui::test]
    fn a_printable_character_jumps_to_the_next_option_that_starts_with_it(cx: &mut TestAppContext) {
        let labels: Vec<SharedString> = ["Apricot", "Banana", "Blueberry", "Cherry"]
            .into_iter()
            .map(SharedString::from)
            .collect();
        let opened = open_labelled(cx, size(px(320.), px(800.)), px(40.), labels, None);

        press(opened.cx, "b");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(1),
            "Banana"
        );
        press(opened.cx, "b");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(2),
            "the same letter again walks to Blueberry rather than sticking on Banana"
        );
        press(opened.cx, "b");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(1),
            "and wraps back round to Banana"
        );

        press(opened.cx, "c");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(3),
            "Cherry"
        );
        press(opened.cx, "a");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(0),
            "a search that runs off the end wraps to the front"
        );

        assert!(
            is_open(&opened.select, opened.cx),
            "type-ahead moves the highlight; it does not choose"
        );
    }

    /// A pointer and a keyboard must not each own a "current row". Hovering
    /// moves the one highlight rather than drawing a second.
    #[gpui::test]
    fn hovering_a_row_moves_the_highlight(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 4, Some(0));

        let row = row_bounds(opened.cx, 2);
        opened
            .cx
            .simulate_mouse_move(row.center(), None, gpui::Modifiers::default());
        opened.cx.run_until_parked();

        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(2),
            "the pointer moved over the third row and the highlight stayed put"
        );
        assert_eq!(
            chosen(&opened.select, opened.cx),
            Some(0),
            "hovering is not choosing"
        );

        press(opened.cx, "down");
        assert_eq!(
            highlighted_row(&opened.select, opened.cx),
            Some(3),
            "the keyboard carries on from where the pointer left the highlight"
        );
    }

    /// gpui `debug_assert!`s if two nodes claim the active descendant in one
    /// frame, so "exactly one" is not a nicety — it is what keeps that panic
    /// unreachable. Read off the declaration rather than the node: the property
    /// is applied at paint time behind gpui's `a11y.is_active()`, which no test
    /// platform here switches on. See `A11y::active_descendant`.
    #[gpui::test]
    fn exactly_one_row_claims_the_active_descendant(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 4, Some(1));
        press(opened.cx, "down");

        let rows: Vec<A11y> = opened.select.read_with(opened.cx, |state, cx| {
            let listbox = state
                .listbox
                .as_ref()
                .expect("the popup should be open")
                .read(cx);
            (0..listbox.options.len())
                .map(|index| listbox.row_a11y(index))
                .collect()
        });

        let claiming: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, a11y)| a11y.is_active_descendant())
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            claiming,
            vec![2],
            "the highlighted row, and only it, is the active descendant"
        );

        // And it is a different row from the chosen one, which is the state the
        // whole change exists to make expressible.
        let selected: Vec<usize> = (0..rows.len())
            .filter(|index| node_for(rows[*index].clone()).is_selected() == Some(true))
            .collect();
        assert_eq!(
            selected,
            vec![1],
            "the choice stayed on row 1 while the highlight moved to row 2"
        );
    }

    /// A listbox with no rows has no highlight, so every key that acts on one
    /// has to do nothing rather than panic or close. Enter is the interesting
    /// one: closing on a key that chose nothing is a worse answer than ignoring
    /// it, and this is the only state that key press is reachable in.
    #[gpui::test]
    fn an_empty_listbox_answers_every_key_by_doing_nothing(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 0, None);

        for key in ["down", "up", "home", "end", "enter", "space", "a"] {
            press(opened.cx, key);
            assert!(
                is_open(&opened.select, opened.cx),
                "{key} closed a popup that had nothing to choose"
            );
            assert_eq!(highlighted_row(&opened.select, opened.cx), None);
            assert_eq!(chosen(&opened.select, opened.cx), None);
        }

        press(opened.cx, "escape");
        assert!(
            !is_open(&opened.select, opened.cx),
            "Escape closes an empty popup — there is always a way out"
        );
    }

    // --- what it announces ---
    //
    // Read off the real `accesskit::Node`, not off an `A11y` value: `A11y`
    // exposes only its role and name, so state could not be asserted on it at
    // all. `a11y::test_support::announced` is no use here either — it takes a
    // `RenderOnce`, and both `SelectState` and `Listbox` are `Render` entities.

    fn node_for(a11y: crate::a11y::A11y) -> gpui::accesskit::Node {
        crate::a11y::test_support::announced_element(div().id("node").announce(a11y))
            .node
            .expect("an id and a role make a node")
    }

    /// The trigger's whole announcement: the *name* is the constructor
    /// argument, the *value* is the chosen option, and `expanded` is on the
    /// control the user operates rather than on the popup.
    #[gpui::test]
    fn the_trigger_announces_a_named_combo_box_valued_by_its_choice(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(1));
        let a11y = opened.select.read_with(opened.cx, |state, _| state.a11y());

        assert_eq!(a11y.role(), Role::ComboBox);
        assert_eq!(a11y.accessible_name(), Some(&"Test select".into()));

        let node = node_for(a11y);
        assert_eq!(node.label(), Some("Test select"));
        assert_eq!(
            node.value(),
            Some("Option 1"),
            "a select's visible text is its value, not its name"
        );
        assert_eq!(node.is_expanded(), Some(true), "the popup is open");
    }

    /// A select with no value reports no value — the placeholder is not one,
    /// and gpui has no `aria_placeholder` in `A11y` to report it as.
    #[gpui::test]
    fn an_unchosen_select_reports_no_value(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let state = cx.update(|cx| {
            cx.new(|_cx| {
                SelectState::new(super::select(
                    "test-select",
                    "Test select",
                    vec![(0usize, "Option 0"), (1usize, "Option 1")],
                ))
            })
        });

        let a11y = state.read_with(cx, |state, _| state.a11y());
        let node = node_for(a11y);

        assert_eq!(node.label(), Some("Test select"));
        assert_eq!(node.value(), None);
        assert_eq!(node.is_expanded(), Some(false));
    }

    /// The interlock with `a11y`'s section 4: `ComboBox` is a role a keyboard
    /// operates, so announcing it without taking focus is the defect that
    /// section exists for. Tab reaches the trigger, and Enter or Space opens
    /// the popup — the popup itself still has no keyboard model.
    #[gpui::test]
    fn the_trigger_takes_keyboard_focus(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let state = cx.update(|cx| {
            cx.new(|_cx| {
                SelectState::new(super::select(
                    "test-select",
                    "Test select",
                    vec![(0usize, "Option 0")],
                ))
            })
        });

        let live = state.read_with(cx, |state, _| state.a11y());
        assert!(live.is_focusable());
        assert!(!live.is_missing_a_focus_decision());

        state.update(cx, |state, cx| state.set_disabled(true, cx));
        let off = state.read_with(cx, |state, _| state.a11y());
        assert!(!off.is_focusable());
        assert!(
            off.focus_declined_because().is_some(),
            "a disabled select leaves the tab order for the same reason a disabled button does, \
             and says so"
        );
    }

    #[gpui::test]
    fn the_popup_is_a_listbox_named_after_its_control(cx: &mut TestAppContext) {
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(0));

        let a11y = opened.select.read_with(opened.cx, |state, cx| {
            state
                .listbox
                .as_ref()
                .expect("the trigger was clicked")
                .read(cx)
                .a11y()
        });

        assert_eq!(a11y.role(), Role::ListBox);
        assert_eq!(node_for(a11y).label(), Some("Test select"));
    }

    /// A row says which one of how many it is. `position_in_set` alone would
    /// announce "2" out of nowhere; `size_of_set` is what makes it "2 of 3".
    #[test]
    fn a_row_announces_its_place_in_the_set() {
        let node = node_for(option_a11y("Option 1".into(), true, false, 1, 3));

        assert_eq!(node.label(), Some("Option 1"));
        assert_eq!(node.is_selected(), Some(true));
        assert_eq!(node.position_in_set(), Some(2), "counted from 1");
        assert_eq!(node.size_of_set(), Some(3));

        let unchosen = node_for(option_a11y("Option 2".into(), false, false, 2, 3));
        assert_eq!(unchosen.is_selected(), Some(false));
        assert_eq!(unchosen.position_in_set(), Some(3));
    }

    #[gpui::test]
    fn a_popup_opened_at_the_bottom_of_the_window_stays_inside_it(cx: &mut TestAppContext) {
        // Eight rows in a 240px window: the listbox fits in the window but
        // neither below the trigger nor above it, so gpui declines to flip and
        // clamps into the window instead. That is the path the margin bug
        // lived on — the clamp was right and the margin then undid it.
        let window = size(px(320.), px(240.));
        let opened = open_select(cx, window, px(120.), 8, Some(0));
        let popup = opened.popup;

        assert!(
            popup.size.height < window.height,
            "the popup is {:?} tall in a {:?}-tall window, so it could not fit however it was \
             placed and this test measures nothing",
            popup.size.height,
            window.height,
        );

        assert!(
            popup.bottom() <= window.height,
            "the popup spans {:?} to {:?} in a {:?}-tall window",
            popup.top(),
            popup.bottom(),
            window.height,
        );
        assert!(
            popup.top() >= px(0.),
            "the popup starts {:?} above the window",
            -popup.top()
        );
    }

    #[gpui::test]
    fn a_popup_hangs_one_gap_below_its_trigger(cx: &mut TestAppContext) {
        // Tall enough that nothing is clamped, so this measures the gap and
        // not the fit.
        let opened = open_select(cx, size(px(320.), px(800.)), px(40.), 3, Some(0));

        let gap = opened.popup.top() - opened.trigger.bottom();
        // One rem at the test window's default rem size. Read from the
        // constant rather than restated, so the two cannot drift.
        let expected = LISTBOX_GAP.to_pixels(px(16.));

        assert!(
            (gap - expected).abs() <= px(1.),
            "the popup hangs {gap:?} below the trigger, expected {expected:?}"
        );
    }

    /// A select with no value is a real state, not an index no option has.
    /// This module used to say "nothing selected" to the popup by passing
    /// `usize::MAX`, which only worked because no list is that long.
    #[gpui::test]
    fn an_unselected_select_marks_no_row_and_a_selected_one_marks_its_own(cx: &mut TestAppContext) {
        let window = size(px(320.), px(800.));

        let opened = open_select(cx, window, px(40.), 3, None);
        assert_eq!(
            marked_row(&opened.select, opened.cx),
            None,
            "a select with no value marked a row anyway"
        );

        let opened = open_select(cx, window, px(40.), 3, Some(2));
        assert_eq!(
            marked_row(&opened.select, opened.cx),
            Some(2),
            "a select holding the third option marked the wrong row"
        );
    }

    /// The row the open listbox is drawing as chosen.
    fn marked_row(select: &Entity<SelectState<usize>>, cx: &VisualTestContext) -> Option<usize> {
        select.read_with(cx, |state, cx| {
            state
                .listbox
                .as_ref()
                .expect("clicking the trigger should have opened the listbox")
                .read(cx)
                .selected_index
        })
    }
}
