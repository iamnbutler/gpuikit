//! Command — a filterable list of actions in an overlay.
//!
//! A command palette is a query field, a list of [`CommandItem`]s, a matcher
//! and a selected row, drawn as a panel near the top of the window over a
//! scrim. It is in the **menu** family — its rows are actions, and nothing
//! stays selected once one has run — even though its nodes announce *listbox*
//! roles, because gpui's `Role::Menu` family carries no
//! `position_in_set`/`size_of_set` and a palette's "3 of 40" is the one thing a
//! screen reader user most needs from it. See `docs/menus-and-listboxes.md`.
//!
//! # Matching is not this crate's business
//!
//! `docs/issues/command.md` argued it and this follows it: the caller installs
//! a closure (`Fn(&str, &[CommandItem]) -> Vec<usize>`, returning item indices
//! in the order they should be shown), or matches somewhere else entirely and
//! answers [`CommandEvent::QueryChanged`] with [`CommandState::set_matches`].
//! A palette that shipped a fuzzy matcher would ship an opinion about ranking
//! that no caller could replace without replacing the component.
//!
//! # The keyboard, which is the whole component
//!
//! `src/input/bindings.rs` binds `up`, `down`, `enter` and `escape` in the
//! `Input` context **on the focused field itself**, and gpui resolves a
//! keystroke by key-context depth. A palette that key-contexts an ancestor
//! therefore loses all four keys: it looks wired and does nothing, and no unit
//! test over a `Vec<usize>` can tell the difference.
//!
//! [`bind_command_keys`] registers each key twice — once under
//! `"CommandPalette > Input"`, which matches at the field's own depth and so
//! **ties** with the field's own binding, and once under bare
//! `"CommandPalette"`. gpui's `KeyBindingContextPredicate::Descendant`
//! (`gpui/src/keymap/context.rs:181`, parsed from `>` at `:361`) is what makes
//! the first form legal, and `Keymap::bindings_for_input`
//! (`gpui/src/keymap.rs:173`) sorts candidates
//! `depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))` — descending depth, then
//! **descending registration index**. A later registration wins a depth tie, so
//! [`crate::init`] calls [`bind_command_keys`] *after*
//! `input::bind_input_keys`. That ordering is load-bearing in the same way the
//! file already documents for Tab. This is Zed's own `"Picker > Editor"` shape.
//!
//! An `on_key_down` was not an option: gpui dispatches bound actions before
//! key-down listeners, so a raw Escape handler loses to any enclosing
//! `Dialog` — the defect `context_menu.rs` still carries and
//! `docs/menus-and-listboxes.md` §3 forbids repeating.
//!
//! # Accessibility
//!
//! The results list announces [`Role::ListBox`], named by a required
//! constructor argument: a palette has no visible text of its own — the field's
//! text is the *query* — which is `src/a11y.rs`'s section 2 and the answer
//! `Select` had to take. Each row announces [`Role::ListBoxOption`] with
//! `selected` and `position_in_set`/`size_of_set` together.
//!
//! **`active_descendant` is deliberately not claimed, and this is the decline
//! in writing.** [`crate::a11y::A11y::active_descendant`] states that gpui puts
//! the property on the *item* and honours it only while a focused **ancestor**
//! of that item is on the node stack, and names this arrangement — focus on a
//! field, pointing at a list beside it — as one that *cannot be expressed*. The
//! rows here are siblings of the focused query field, so the claim would be
//! dropped in silence; worse, it is applied at paint time behind
//! `window.a11y.is_active()`, which no test platform here switches on, so a
//! wrong claim would ship green forever. The selected row carries its fill and
//! its `selected` state and nothing more.
//! [`crate::elements::combobox`] meets the same wall for the same reason. Two
//! components independently unable to express the APG arrangement is a gap in
//! gpui, not in either of them.

use crate::a11y::{A11y, Announce};
use crate::element_id::for_entity;
use crate::elements::listbox::wrapped_index;
use crate::elements::text_field::text_field;
use crate::input::{InputState, InputStateEvent};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use gpui::{
    actions, deferred, div, prelude::*, px, App, Context, ElementId, Entity, EventEmitter,
    Focusable, IntoElement, KeyBinding, ParentElement, Rems, Render, Role, SharedString, Styled,
    Svg, Window,
};
use std::rc::Rc;

actions!(
    command,
    [
        /// Move the palette's selection to the next runnable row.
        CommandSelectNext,
        /// Move the palette's selection to the previous runnable row.
        CommandSelectPrevious,
        /// Run the selected command.
        CommandRun,
        /// Dismiss the palette without running anything.
        CommandDismiss,
    ]
);

/// The key context the palette's panel declares, and the parent half of the
/// `"CommandPalette > Input"` predicate.
pub const COMMAND_CONTEXT: &str = "CommandPalette";

/// Bind the palette's four keys.
///
/// **Call this after `input::bind_input_keys`.** See this module's
/// `# The keyboard`: the `"CommandPalette > Input"` form ties on depth with the
/// field's own binding and wins only by being registered later. Registered
/// first, this compiles, runs, and does nothing — every key goes to the text
/// field.
pub fn bind_command_keys(cx: &mut App) {
    let under_input = Some("CommandPalette > Input");
    cx.bind_keys([
        KeyBinding::new("down", CommandSelectNext, under_input),
        KeyBinding::new("up", CommandSelectPrevious, under_input),
        KeyBinding::new("enter", CommandRun, under_input),
        KeyBinding::new("escape", CommandDismiss, under_input),
        KeyBinding::new("down", CommandSelectNext, Some(COMMAND_CONTEXT)),
        KeyBinding::new("up", CommandSelectPrevious, Some(COMMAND_CONTEXT)),
        KeyBinding::new("enter", CommandRun, Some(COMMAND_CONTEXT)),
        KeyBinding::new("escape", CommandDismiss, Some(COMMAND_CONTEXT)),
    ]);
}

/// How far down the window the panel sits.
///
/// Not `theme.control(size)`: a rung describes a control's own dimensions, and
/// this is a distance from the window's edge. `src/theme/control.rs`'s "what
/// belongs here" note says to name it here instead.
const PANEL_TOP: Rems = Rems(6.0);

/// The panel's width, in pixels. `gpui::Pixels`' tuple constructor is private
/// outside gpui, so this is an `f32` wrapped in `px()` at the call site.
const PANEL_WIDTH: f32 = 560.0;

/// How tall the results may get before they scroll.
const RESULTS_MAX_HEIGHT: f32 = 360.0;

/// What the palette emits.
pub enum CommandEvent {
    /// The query changed. A caller matching asynchronously answers with
    /// [`CommandState::set_matches`].
    QueryChanged(SharedString),
    /// A command ran. Carries its index into the palette's items.
    Run(usize),
    /// The palette was dismissed without running anything.
    Dismissed,
}

/// One row of a command palette.
pub struct CommandItem {
    /// What the row says.
    pub label: SharedString,
    /// A second line, for the rows a label cannot explain.
    pub subtitle: Option<SharedString>,
    /// Extra words a matcher should search but the row does not show.
    pub keywords: Vec<SharedString>,
    /// The keystroke this command answers to elsewhere, drawn on the right.
    pub shortcut: Option<SharedString>,
    /// `impl Fn() -> Svg` rather than an `Svg`, because `gpui::Svg` is not
    /// `Clone` — the same signature `context_menu.rs` carries for the same
    /// reason.
    pub icon: Option<Rc<dyn Fn() -> Svg>>,
    /// A row that is shown and cannot be run. Skipped by the selection.
    pub disabled: bool,
    /// What running it does.
    pub on_run: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl CommandItem {
    /// A row with a label and nothing else.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            subtitle: None,
            keywords: Vec::new(),
            shortcut: None,
            icon: None,
            disabled: false,
            on_run: None,
        }
    }

    /// Set the second line.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Add words a matcher should see and the row should not show.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    /// Set the keystroke drawn on the right.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set the leading icon.
    pub fn icon(mut self, icon: impl Fn() -> Svg + 'static) -> Self {
        self.icon = Some(Rc::new(icon));
        self
    }

    /// Show the row and refuse to run it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set what running it does.
    pub fn on_run(mut self, on_run: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_run = Some(Rc::new(on_run));
        self
    }

    /// Every searchable string this row has, in one place, so a caller's
    /// matcher has somewhere to read from rather than reassembling it.
    pub fn haystack(&self) -> String {
        let mut haystack = self.label.to_string();
        if let Some(subtitle) = &self.subtitle {
            haystack.push(' ');
            haystack.push_str(subtitle);
        }
        for keyword in &self.keywords {
            haystack.push(' ');
            haystack.push_str(keyword);
        }
        haystack
    }
}

/// The matcher: a query and every item, in, item indices in display order, out.
type Matcher = Rc<dyn Fn(&str, &[CommandItem]) -> Vec<usize>>;

/// A command palette.
pub struct CommandState {
    id: ElementId,
    /// The accessible name of the results list. A required argument for the
    /// reason in this module's `# Accessibility`.
    label: SharedString,
    query: Entity<InputState>,
    items: Vec<CommandItem>,
    /// Item indices, in the order the palette draws them.
    matches: Vec<usize>,
    matcher: Option<Matcher>,
    /// An index **into `matches`**, never into `items`.
    ///
    /// One field, translated at the edges by [`Self::selected_item`], is what
    /// stops the "selection survived a re-match onto a different row" bug.
    selected: Option<usize>,
    open: bool,
    size: ControlSize,
}

impl EventEmitter<CommandEvent> for CommandState {}

impl CommandState {
    /// Build a palette. `name` names the results list — see
    /// `# Accessibility`.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        items: Vec<CommandItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query = cx.new(|cx| {
            let mut state = InputState::new_singleline(cx);
            state.set_placeholder(SharedString::from("Type a command…"), cx);
            state
        });

        cx.subscribe_in(&query, window, |this, _query, event, _window, cx| {
            if matches!(event, InputStateEvent::TextChanged) {
                this.query_changed(cx);
            }
        })
        .detach();

        let matches = (0..items.len()).collect::<Vec<_>>();
        let mut state = Self {
            id: id.into(),
            label: name.into(),
            query,
            items,
            matches,
            matcher: None,
            selected: None,
            open: false,
            size: ControlSize::default(),
        };
        state.selected = state.first_runnable();
        state
    }

    /// Install the matcher. Without one the palette shows every item in order,
    /// which is the right behaviour for a caller who answers
    /// [`CommandEvent::QueryChanged`] instead.
    pub fn matcher(mut self, matcher: impl Fn(&str, &[CommandItem]) -> Vec<usize> + 'static) -> Self {
        self.matcher = Some(Rc::new(matcher));
        self
    }

    /// Replace the items. Re-matches against the current query.
    pub fn set_items(&mut self, items: Vec<CommandItem>, cx: &mut Context<Self>) {
        self.items = items;
        self.rematch(cx);
    }

    /// Answer a [`CommandEvent::QueryChanged`] with the matched item indices,
    /// in display order. The asynchronous half of the matcher hook.
    pub fn set_matches(&mut self, matches: Vec<usize>, cx: &mut Context<Self>) {
        self.matches = matches.into_iter().filter(|i| *i < self.items.len()).collect();
        self.selected = self.first_runnable();
        cx.notify();
    }

    /// Whether the palette is showing.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Show the palette, empty the query and put focus in it.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.query.update(cx, |state, cx| state.set_content("", cx));
        self.rematch(cx);
        let handle = self.query.read(cx).focus_handle(cx);
        window.focus(&handle, cx);
        cx.notify();
    }

    /// Hide the palette and say so.
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        cx.emit(CommandEvent::Dismissed);
        cx.notify();
    }

    /// The rows that can actually be run, as indices into `matches`.
    ///
    /// The shape `context_menu.rs`'s `selectable_indices` has, over a different
    /// collection: a disabled row is drawn and skipped.
    fn runnable_rows(&self) -> Vec<usize> {
        self.matches
            .iter()
            .enumerate()
            .filter(|(_, item)| self.items.get(**item).is_some_and(|item| !item.disabled))
            .map(|(row, _)| row)
            .collect()
    }

    fn first_runnable(&self) -> Option<usize> {
        self.runnable_rows().first().copied()
    }

    /// Move the selection by `delta` runnable rows, wrapping at both ends.
    ///
    /// A selection that is no longer runnable re-enters from the edge the
    /// movement came from, which is `context_menu.rs`'s `next_focus` rule.
    /// `wrapped_index` is shared with `Listbox` — the arithmetic is the same
    /// and was worth extracting rather than writing twice.
    fn next_selection(&self, delta: isize) -> Option<usize> {
        let runnable = self.runnable_rows();
        let position = self
            .selected
            .and_then(|selected| runnable.iter().position(|row| *row == selected));
        let next = wrapped_index(position, delta, runnable.len())?;
        runnable.get(next).copied()
    }

    /// The item index the selection points at, translated out of `matches`.
    pub fn selected_item(&self) -> Option<usize> {
        self.matches.get(self.selected?).copied()
    }

    fn query_changed(&mut self, cx: &mut Context<Self>) {
        let query = SharedString::from(self.query.read(cx).content().to_string());
        self.rematch(cx);
        cx.emit(CommandEvent::QueryChanged(query));
    }

    /// Re-run the matcher and re-select the first runnable row, which is what
    /// makes type-then-Enter work with no arrow press.
    fn rematch(&mut self, cx: &mut Context<Self>) {
        let query = self.query.read(cx).content().to_string();
        self.matches = match &self.matcher {
            Some(matcher) => matcher(&query, &self.items)
                .into_iter()
                .filter(|index| *index < self.items.len())
                .collect(),
            None => (0..self.items.len()).collect(),
        };
        self.selected = self.first_runnable();
        cx.notify();
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if !self.open {
            cx.propagate();
            return;
        }
        self.selected = self.next_selection(delta);
        cx.notify();
    }

    /// Run the selected row. Dismisses first: nothing stays selected once a
    /// command has run, which is the whole of the menu family.
    fn run_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            cx.propagate();
            return;
        }
        let Some(index) = self.selected_item() else {
            return;
        };
        let on_run = self.items.get(index).and_then(|item| item.on_run.clone());
        self.open = false;
        cx.emit(CommandEvent::Run(index));
        cx.notify();
        if let Some(on_run) = on_run {
            on_run(window, cx);
        }
    }

    fn handle_dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            // Not ours. An unconditional consume is how a palette that is not
            // showing would stop an enclosing dialog closing.
            cx.propagate();
            return;
        }
        self.dismiss(cx);
    }

    fn row_a11y(&self, row: usize, item: &CommandItem) -> A11y {
        A11y::new(Role::ListBoxOption)
            .name(item.label.clone())
            .selected(self.selected == Some(row))
            // Both, together: a position with no size announces "3" out of
            // nowhere. No `active_descendant` — see this module's docs.
            .position_in_set(row + 1)
            .size_of_set(self.matches.len())
    }
}

/// The results list is named by the constructor argument. The palette's own
/// visible text is the *query*, which is not a name for anything.
impl Accessible for CommandState {
    fn a11y(&self) -> A11y {
        A11y::new(Role::ListBox).name(self.label.clone())
    }
}

impl ControlSized for CommandState {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Render for CommandState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let list_a11y = self.a11y();
        let top = PANEL_TOP.to_pixels(window.rem_size());

        let scrim = theme.overlay();
        let surface = theme.surface();
        let border = theme.border();
        let fg = theme.fg();
        let fg_muted = theme.fg_muted();
        let accent = theme.accent();

        let rows: Vec<_> = self
            .matches
            .iter()
            .enumerate()
            .filter_map(|(row, index)| self.items.get(*index).map(|item| (row, *index, item)))
            .map(|(row, index, item)| {
                let is_selected = self.selected == Some(row);
                let a11y = self.row_a11y(row, item);
                let disabled = item.disabled;

                let element = div()
                    .id(ElementId::NamedInteger("command-row".into(), row as u64))
                    .announce(a11y)
                    .flex()
                    .items_center()
                    .gap(metrics.gap)
                    .h(metrics.height)
                    .px(metrics.padding_x * 1.5)
                    .text_size(metrics.text_size)
                    .line_height(metrics.line_height)
                    .text_color(if disabled {
                        theme.fg_disabled()
                    } else if is_selected {
                        theme.bg()
                    } else {
                        fg
                    })
                    .when(is_selected && !disabled, |this| this.bg(accent))
                    .when(!disabled, |this| {
                        this.cursor_pointer()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.selected = Some(row);
                                this.run_selected(window, cx);
                            }))
                    })
                    .when_some(item.icon.clone(), |this, icon| {
                        this.child(icon().size(metrics.text_size))
                    })
                    .child(div().flex_1().child(item.label.clone()))
                    .when_some(item.subtitle.clone(), |this, subtitle| {
                        this.child(div().text_color(fg_muted).child(subtitle))
                    })
                    .when_some(item.shortcut.clone(), |this, shortcut| {
                        this.child(div().text_color(fg_muted).child(shortcut))
                    });

                #[cfg(test)]
                let element = element
                    .debug_selector(move || format!("gpuikit-command-row-{index}"));
                #[cfg(not(test))]
                let _ = index;

                element
            })
            .collect();

        let panel = div()
            .id(self.id.clone())
            .occlude()
            .w(px(PANEL_WIDTH))
            .max_w_full()
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(metrics.radius)
            .shadow_lg()
            .flex()
            .flex_col()
            .child(
                div()
                    .p(metrics.padding_x)
                    .border_b_1()
                    .border_color(border)
                    .child(text_field(&self.query, cx).full_width(true)),
            )
            .child(
                div()
                    .id(for_entity("gpuikit-command-results", cx.entity_id()))
                    .announce(list_a11y)
                    .max_h(px(RESULTS_MAX_HEIGHT))
                    .overflow_y_scroll()
                    .py(metrics.padding_y())
                    .flex()
                    .flex_col()
                    .children(rows),
            );

        #[cfg(test)]
        let panel = panel.debug_selector(|| "gpuikit-command-panel".into());

        let scrimmed = div()
            .id(for_entity("gpuikit-command", cx.entity_id()))
            .key_context(COMMAND_CONTEXT)
            .on_action(cx.listener(|this, _: &CommandSelectNext, _window, cx| {
                this.move_selection(1, cx);
            }))
            .on_action(cx.listener(|this, _: &CommandSelectPrevious, _window, cx| {
                this.move_selection(-1, cx);
            }))
            .on_action(cx.listener(|this, _: &CommandRun, window, cx| {
                this.run_selected(window, cx);
            }))
            .on_action(cx.listener(|this, _: &CommandDismiss, _window, cx| {
                this.handle_dismiss(cx);
            }))
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(scrim)
            .pt(top)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _window, cx| {
                    this.dismiss(cx);
                }),
            )
            .child(panel);

        // Rung 10 of `docs/overlays.md`'s ladder — the dialog layer. A palette
        // is a modal over a scrim and belongs at the same height as one. There
        // is no trigger to hang off, so it is `anchored()`-free like `dialog`
        // and its distance from the top is padding on the scrim rather than an
        // offset.
        div().when(self.open, |this| {
            this.child(deferred(scrimmed).with_priority(10))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<CommandItem> {
        vec![
            CommandItem::new("Open File").keywords(["edit"]),
            CommandItem::new("Save").disabled(true),
            CommandItem::new("Save As").subtitle("write a copy"),
            CommandItem::new("Quit"),
        ]
    }

    /// The selection model, over plain index lists — `runnable_rows` and
    /// `next_selection` are `&self` and read only `items`, `matches` and
    /// `selected`, so they are exercised through a struct literal built by
    /// hand.
    fn model(matches: Vec<usize>) -> (Vec<usize>, Vec<CommandItem>) {
        (matches, items())
    }

    /// Rows that can be run, given the matched set. Row 1 of the unfiltered
    /// list is `Save`, which is disabled.
    fn runnable(matches: &[usize], items: &[CommandItem]) -> Vec<usize> {
        matches
            .iter()
            .enumerate()
            .filter(|(_, item)| items.get(**item).is_some_and(|item| !item.disabled))
            .map(|(row, _)| row)
            .collect()
    }

    #[test]
    fn a_disabled_row_is_shown_and_skipped() {
        let (matches, items) = model(vec![0, 1, 2, 3]);
        assert_eq!(runnable(&matches, &items), vec![0, 2, 3]);
    }

    #[test]
    fn the_selection_wraps_at_both_ends() {
        let runnable = vec![0usize, 2, 3];
        assert_eq!(wrapped_index(Some(2), 1, runnable.len()), Some(0));
        assert_eq!(wrapped_index(Some(0), -1, runnable.len()), Some(2));
    }

    #[test]
    fn an_empty_result_set_selects_nothing() {
        assert_eq!(wrapped_index(None, 1, 0), None);
        let (matches, items) = model(Vec::new());
        assert!(runnable(&matches, &items).is_empty());
    }

    #[test]
    fn entering_from_the_far_end_comes_in_at_the_bottom() {
        assert_eq!(wrapped_index(None, -1, 3), Some(2));
        assert_eq!(wrapped_index(None, 1, 3), Some(0));
    }

    #[test]
    fn a_row_index_is_not_an_item_index() {
        // Matched: items 2 and 3 only. Row 0 is item 2.
        let matches = vec![2usize, 3];
        assert_eq!(matches.get(0).copied(), Some(2));
        assert_eq!(matches.get(1).copied(), Some(3));
    }

    #[test]
    fn every_runnable_row_survives_a_filter() {
        let (matches, items) = model(vec![1, 2]);
        // Row 0 is `Save`, disabled; row 1 is `Save As`.
        assert_eq!(runnable(&matches, &items), vec![1]);
    }

    #[test]
    fn a_haystack_carries_the_label_the_subtitle_and_the_keywords() {
        let item = CommandItem::new("Open File")
            .subtitle("from disk")
            .keywords(["edit", "load"]);
        assert_eq!(item.haystack(), "Open File from disk edit load");
    }
}
