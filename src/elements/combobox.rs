//! Combobox — a text field that filters a list of choices.
//!
//! A combobox is a [`TextField`](crate::elements::text_field) that owns a
//! listbox. The typed text filters the options, the arrow keys move a highlight
//! through what survived the filter, and Enter commits the highlighted option
//! to the control's **value**. It is in the listbox family, not the menu
//! family: its rows are values and the choice persists. See
//! `docs/menus-and-listboxes.md`.
//!
//! # Two things, not one
//!
//! The state holds the **value** ([`ComboboxState::selected`]) and the **text**
//! (an `Entity<InputState>`) separately, because they diverge the moment the
//! user types. The text is not duplicated into a `String` field of our own:
//! `InputState` already owns editing, IME and selection, and a second copy
//! would be two sources of truth for one string.
//!
//! Typing **clears the value**. A value the text no longer names is a lie the
//! control would go on reporting through `on_change` and through its
//! accessible value. What happens to text that matches nothing when the field
//! loses focus is the caller's decision, spelled [`UnmatchedText`], and
//! `Revert` is the default: a combobox is a chooser, its value is the thing
//! that persists, and a field left showing text that is not the value is the
//! bug. Note where those two rules meet: typing clears the value, so `Revert`
//! after typing something unmatched empties the field rather than restoring
//! what was there before. That is the rule followed to its end — the field
//! shows the value, and there is no value — and it is why `Keep` and `Create`
//! exist for the callers that want the other answers.
//!
//! # The keyboard, which is the hard part
//!
//! Every key this needs — `up`, `down`, `enter`, `escape` — is already bound by
//! `src/input/bindings.rs` in `INPUT_CONTEXT`, **on the focused field itself**.
//! `Select`'s answer does not transfer: a select moves focus *into* its popup
//! and binds in the deeper `Listbox` context, which a combobox cannot do
//! without stopping the user typing.
//!
//! Three routes were available and two are refuted:
//!
//! * A raw `on_key_down` is forbidden. gpui dispatches bound actions before
//!   key-down listeners, so an enclosing `Dialog`'s Escape binding would take
//!   Escape first. `select.rs`'s module docs and `docs/menus-and-listboxes.md`
//!   §3 both record this as the defect `context_menu.rs` still has.
//! * An ancestor `on_action` listener does not work either. gpui clears
//!   `propagate_event` before every bubble-phase listener, so an action the
//!   focused input handles is swallowed unless the input calls `cx.propagate()`
//!   — and `InputState::copy` is the crate's only such call. Nothing further
//!   out on the focus path would ever see these four keys.
//! * What does work is **binding depth**. gpui's
//!   `KeyBindingContextPredicate::Descendant` (`gpui/src/keymap/context.rs`,
//!   parsed from `>`) lets a binding say *"`Input`, but only under a
//!   `Combobox`"*. It matches at the input's own node, so it **ties** with
//!   `bind_input_keys`' plain `Input` binding on depth — and
//!   `Keymap::bindings_for_input` (`gpui/src/keymap.rs`) sorts
//!   `depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))`, descending registration index
//!   last. **The later registration wins the tie**, which is why
//!   [`crate::init`] calls [`bind_combobox_keys`] after `bind_input_keys`. That
//!   ordering is load-bearing in the same way the file already documents for
//!   Tab.
//!
//! `home`, `end`, `left` and `right` are deliberately **not** taken: they are
//! text navigation and belong to the field. That is the difference between this
//! and `Listbox`'s own key map, and it is why these bindings live here rather
//! than being inherited.
//!
//! Each of the four actions calls `cx.propagate()` when the popup is closed and
//! it has nothing to do — the [`InputState::copy`] shape. So Escape in a
//! combobox inside a dialog closes the popup if one is open and the dialog if
//! not, which is the behaviour a user expects and the one an unconditional
//! consume would break.
//!
//! # Accessibility
//!
//! The wrapper announces [`Role::ComboBox`] with the constructor's name, its
//! `expanded` state and — as its value — the text currently in the field. It
//! goes on the wrapper rather than on the `TextField`, because `TextField`
//! reports no role of its own and teaching it one would give a role to every
//! plain text field in the crate.
//!
//! **`active_descendant` is deliberately not claimed, and this is the decline
//! in writing.** [`crate::a11y::A11y::active_descendant`] states that gpui puts
//! the property on the *item* and honours it only while a focused **ancestor**
//! of that item is on the node stack, and names this exact arrangement — focus
//! on a combo box, pointing into a popup beside it — as one that *cannot be
//! expressed*. The popup here is a sibling subtree under `div().relative()`,
//! not a child of the focused input, so the claim would be dropped in silence
//! and no test could read it back. The highlighted row therefore carries its
//! fill and its `selected` state and nothing more. This is a gap in gpui, not
//! in this element; `crate::elements::command` meets the same wall.
//!
//! # Not built
//!
//! * The popup does not match the field's width. `docs/overlays.md` says the
//!   thing to build is a small custom `Element` that measures a trigger, *when
//!   a second element wants it*. This is the first. It keeps `Listbox`'s
//!   `min_w`.
//! * Multiple selection. It turns `selected: Option<T>` into a set, the text
//!   into a query that is not the value, and the blur rule into something else
//!   entirely. It is a second issue, and nothing here prevents it.

use crate::a11y::{A11y, Announce};
use crate::elements::listbox::{matches_query, Listbox, ListboxFocus, LISTBOX_GAP};
use crate::elements::text_field::{text_field, Adornment};
use crate::icons::Icons;
use crate::input::{InputState, InputStateEvent};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    actions, anchored, deferred, div, point, prelude::*, px, App, Context, DismissEvent, ElementId,
    Entity, EventEmitter, IntoElement, KeyBinding, ParentElement, Render, Role, SharedString,
    Styled, Window,
};
use std::rc::Rc;

actions!(
    combobox,
    [
        /// Move the highlight to the next visible option, opening the popup
        /// first if it is closed.
        ComboboxHighlightNext,
        /// Move the highlight to the previous visible option, opening the
        /// popup first if it is closed.
        ComboboxHighlightPrevious,
        /// Commit the highlighted option to the value.
        ComboboxChoose,
        /// Close the popup without choosing anything.
        ComboboxDismiss,
    ]
);

/// The key context the combobox's wrapper declares.
///
/// Public because the bindings are, and because the bindings name it twice —
/// once alone and once as the parent half of `"Combobox > Input"`.
pub const COMBOBOX_CONTEXT: &str = "Combobox";

/// Bind the combobox's four keys: Down, Up, Enter and Escape.
///
/// **Call this after `input::bind_input_keys`.** Every one of these keys is
/// also bound in the plain `Input` context on the field that holds focus. The
/// `"Combobox > Input"` predicate matches at the same node, so it ties on
/// depth, and gpui breaks a depth tie by *later registration*. Registered
/// first, this function would compile, run, and do nothing at all — the arrow
/// keys would move the text cursor. See this module's `# The keyboard`.
///
/// The bare `COMBOBOX_CONTEXT` form is registered too, for the frame where the
/// wrapper is on the dispatch path but the field is not focused.
///
/// `home`, `end`, `left` and `right` are deliberately absent: a combobox is a
/// text field first, and those four are how a user moves through what they
/// typed.
pub fn bind_combobox_keys(cx: &mut App) {
    let under_input = Some("Combobox > Input");
    cx.bind_keys([
        KeyBinding::new("down", ComboboxHighlightNext, under_input),
        KeyBinding::new("up", ComboboxHighlightPrevious, under_input),
        KeyBinding::new("enter", ComboboxChoose, under_input),
        KeyBinding::new("escape", ComboboxDismiss, under_input),
        KeyBinding::new("down", ComboboxHighlightNext, Some(COMBOBOX_CONTEXT)),
        KeyBinding::new("up", ComboboxHighlightPrevious, Some(COMBOBOX_CONTEXT)),
        KeyBinding::new("enter", ComboboxChoose, Some(COMBOBOX_CONTEXT)),
        KeyBinding::new("escape", ComboboxDismiss, Some(COMBOBOX_CONTEXT)),
    ]);
}

/// Emitted when the combobox's **value** changes — including when typing
/// clears it, which is a change like any other.
pub struct ComboboxChanged;

/// What to do with text that matches no option when the field loses focus.
///
/// Three answers, because there is no one right one, and an enum rather than a
/// fork of this file because the two non-default answers are options a caller
/// picks rather than behaviours a crate chooses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnmatchedText {
    /// Put back the label of the chosen option, or empty the field when there
    /// is none. **The default.** A combobox is a chooser; its value is the
    /// thing that persists.
    #[default]
    Revert,
    /// Leave the text and leave the value cleared. For a field whose list is a
    /// suggestion rather than a constraint.
    Keep,
    /// Hand the text to the `on_create` handler. Set by calling
    /// [`Combobox::on_create`], so the mode cannot be chosen without the
    /// handler that makes it mean anything. The handler decides whether to push
    /// an option; the combobox does not mutate its own option list behind the
    /// caller's back.
    Create,
}

/// The filter a combobox runs: query, value, label.
type Filter<T> = Rc<dyn Fn(&str, &T, &SharedString) -> bool>;

/// Builder for a combobox. Use [`combobox`] to create one.
pub struct Combobox<T: Clone + PartialEq + 'static> {
    id: ElementId,
    label: SharedString,
    options: Vec<(T, SharedString)>,
    selected: Option<T>,
    placeholder: SharedString,
    on_change: Option<Rc<dyn Fn(Option<T>, &mut Window, &mut App)>>,
    on_create: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
    filter: Filter<T>,
    unmatched: UnmatchedText,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

/// Creates a new combobox builder.
///
/// # Arguments
///
/// * `id` — unique identifier for the combobox
/// * `name` — the accessible name: what this control is *for*, as distinct
///   from what is currently chosen in it. Required for the same reason
///   [`crate::elements::select::select`]'s is: [`Role::ComboBox`] is in
///   [`crate::a11y::role_requires_a_name`], the visible text is the *value*,
///   and gpui has no `labelled_by`
/// * `options` — vector of (value, label) tuples
pub fn combobox<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    options: Vec<(T, impl Into<SharedString>)>,
) -> Combobox<T> {
    Combobox::new(id, name, options)
}

impl<T: Clone + PartialEq + 'static> Combobox<T> {
    /// See [`combobox`].
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        options: Vec<(T, impl Into<SharedString>)>,
    ) -> Self {
        Self {
            id: id.into(),
            label: name.into(),
            options: options
                .into_iter()
                .map(|(value, label)| (value, label.into()))
                .collect(),
            selected: None,
            placeholder: "Search…".into(),
            on_change: None,
            on_create: None,
            filter: Rc::new(|query, _value, label| matches_query(query, label)),
            unmatched: UnmatchedText::default(),
            full_width: false,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    /// Set the initially selected value. Its label becomes the field's text.
    pub fn selected(mut self, value: T) -> Self {
        self.selected = Some(value);
        self
    }

    /// Set the placeholder shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Register a callback for when the **value** changes. `None` is a real
    /// change: it is what typing does.
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<T>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Replace the match. Query, value, label — the label is what the default
    /// looks at, and the value is there so a caller can match on something the
    /// row does not show.
    pub fn filter(mut self, filter: impl Fn(&str, &T, &SharedString) -> bool + 'static) -> Self {
        self.filter = Rc::new(filter);
        self
    }

    /// Leave unmatched text in the field on blur — [`UnmatchedText::Keep`].
    pub fn keep_unmatched_text(mut self) -> Self {
        self.unmatched = UnmatchedText::Keep;
        self
    }

    /// Hand unmatched text to `handler` on blur — [`UnmatchedText::Create`].
    ///
    /// Setting the handler is what selects the mode, so there is no way to ask
    /// for `Create` and not answer it.
    pub fn on_create(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_create = Some(Rc::new(handler));
        self.unmatched = UnmatchedText::Create;
        self
    }

    /// Make the combobox expand to fill available width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

impl<T: Clone + PartialEq + 'static> Disableable for Combobox<T> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Clone + PartialEq + 'static> ControlSized for Combobox<T> {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

/// The stateful combobox. Build one with [`ComboboxState::new`] inside
/// `cx.new(…)`; it has to outlive a frame because it owns two entities.
pub struct ComboboxState<T: Clone + PartialEq + 'static> {
    id: ElementId,
    label: SharedString,
    options: Vec<(T, SharedString)>,
    /// The chosen value. `None` while the text names nothing.
    pub selected: Option<T>,
    /// The text. `InputState` owns editing, IME and selection; this element
    /// owns the value, and the two are deliberately not the same field.
    input: Entity<InputState>,
    listbox: Option<Entity<Listbox>>,
    /// Indices into `options` that survived the filter, in the order the popup
    /// draws them.
    ///
    /// **The popup's row index is not the option index.** `Listbox` reports the
    /// row it was clicked or Entered on; this is how that gets back to an
    /// option. Getting it wrong picks the wrong value *only when a filter is
    /// active*, which is exactly the case a smoke test never reaches.
    visible: Vec<usize>,
    on_change: Option<Rc<dyn Fn(Option<T>, &mut Window, &mut App)>>,
    on_create: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
    filter: Filter<T>,
    unmatched: UnmatchedText,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<ComboboxChanged> for ComboboxState<T> {}

impl<T: Clone + PartialEq + 'static> ComboboxState<T> {
    /// Build the state, its `InputState` and the subscription that ties the two
    /// together.
    pub fn new(combobox: Combobox<T>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let placeholder = combobox.placeholder.clone();
        let input = cx.new(|cx| {
            let mut state = InputState::new_singleline(cx);
            state.set_placeholder(placeholder, cx);
            state
        });

        // The field starts showing the label of whatever was chosen, so text
        // and value agree from the first frame.
        if let Some(selected) = &combobox.selected {
            if let Some((_, label)) = combobox.options.iter().find(|(v, _)| v == selected) {
                let label = label.to_string();
                input.update(cx, |state, cx| state.set_content(label, cx));
            }
        }

        cx.subscribe_in(
            &input,
            window,
            |this, _input, event, window, cx| match event {
                InputStateEvent::TextChanged => this.text_changed(window, cx),
                InputStateEvent::Blur => this.blurred(window, cx),
                _ => {}
            },
        )
        .detach();

        let visible = (0..combobox.options.len()).collect();

        Self {
            id: combobox.id,
            label: combobox.label,
            options: combobox.options,
            selected: combobox.selected,
            input,
            listbox: None,
            visible,
            on_change: combobox.on_change,
            on_create: combobox.on_create,
            filter: combobox.filter,
            unmatched: combobox.unmatched,
            full_width: combobox.full_width,
            disabled: combobox.disabled,
            size: combobox.size,
        }
    }

    /// The text currently in the field.
    pub fn text(&self, cx: &App) -> SharedString {
        self.input.read(cx).content().to_string().into()
    }

    /// Whether the popup is open.
    pub fn is_open(&self) -> bool {
        self.listbox.is_some()
    }

    /// The label of the chosen option, if there is one.
    fn selected_label(&self) -> Option<SharedString> {
        let selected = self.selected.as_ref()?;
        self.options
            .iter()
            .find(|(value, _)| value == selected)
            .map(|(_, label)| label.clone())
    }

    /// Recompute [`Self::visible`] for `query`, and hand the survivors' labels
    /// back for the popup.
    ///
    /// The row index and the option index part company here and nowhere else,
    /// which is why this is one function rather than a filter inlined at each
    /// call site.
    fn refilter(&mut self, query: &str) -> Vec<SharedString> {
        let filter = self.filter.clone();
        self.visible = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, (value, label))| filter(query, value, label))
            .map(|(index, _)| index)
            .collect();
        self.visible
            .iter()
            .map(|index| self.options[*index].1.clone())
            .collect()
    }

    /// Which *row* of the popup carries the selection, translated from the
    /// option index. `None` when the chosen option did not survive the filter.
    fn selected_row(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        let option_index = self
            .options
            .iter()
            .position(|(value, _)| value == selected)?;
        self.visible.iter().position(|index| *index == option_index)
    }

    /// Set the value and tell everyone, including when the value is `None`.
    fn set_value(&mut self, value: Option<T>, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected == value {
            return;
        }
        self.selected = value.clone();
        if let Some(on_change) = self.on_change.clone() {
            on_change(value, window, cx);
        }
        cx.emit(ComboboxChanged);
        cx.notify();
    }

    /// Commit the option at *row* `row` — a popup row, not an option index.
    fn choose_row(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(option_index) = self.visible.get(row).copied() else {
            return;
        };
        let Some((value, label)) = self.options.get(option_index).cloned() else {
            return;
        };
        let label_text = label.to_string();
        self.input
            .update(cx, |state, cx| state.set_content(label_text, cx));
        self.set_value(Some(value), window, cx);
        self.listbox = None;
        // Committing rebuilds the unfiltered list, so the next press of Down
        // offers every option rather than the one that happened to match the
        // text that is now the value.
        self.refilter("");
        cx.notify();
    }

    /// Typing. Re-filter, open, reset the highlight, and clear the value.
    fn text_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let query = self.text(cx).to_string();
        let labels = self.refilter(&query);
        // A value the text no longer names is a lie this control would keep
        // reporting. Cleared here, and `ComboboxChanged` is emitted for it.
        self.set_value(None, window, cx);

        match self.listbox.clone() {
            Some(listbox) => {
                listbox.update(cx, |listbox, cx| listbox.set_options(labels, None, cx));
            }
            None => self.open(window, cx),
        }
        cx.notify();
    }

    /// Losing focus. [`UnmatchedText`] decides.
    fn blurred(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.listbox = None;
        let text = self.text(cx);

        if self.selected_label().as_ref() == Some(&text) {
            cx.notify();
            return;
        }

        match self.unmatched {
            UnmatchedText::Revert => {
                let restored = self.selected_label().unwrap_or_default().to_string();
                self.input
                    .update(cx, |state, cx| state.set_content(restored, cx));
            }
            UnmatchedText::Keep => {}
            UnmatchedText::Create => {
                if let Some(on_create) = self.on_create.clone() {
                    if !text.is_empty() {
                        on_create(text, window, cx);
                    }
                }
            }
        }
        cx.notify();
    }

    /// Open the popup over the current filter.
    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.listbox.is_some() {
            return;
        }
        let labels: Vec<SharedString> = self
            .visible
            .iter()
            .map(|index| self.options[*index].1.clone())
            .collect();
        let selected_row = self.selected_row();
        let entity = cx.entity().downgrade();

        let listbox = Listbox::build(
            self.label.clone(),
            labels,
            selected_row,
            self.size,
            // Focus stays on the text field. Everything in this module's
            // `# The keyboard` follows from this line.
            ListboxFocus::Caller,
            move |row, window, cx| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |this, cx| this.choose_row(row, window, cx));
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

    /// Down and Up. Opens the popup when it is closed, so the first press of
    /// Down is how a combobox is opened from the keyboard.
    fn move_highlight(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            cx.propagate();
            return;
        }
        match self.listbox.clone() {
            Some(listbox) => listbox.update(cx, |listbox, cx| listbox.move_highlight(delta, cx)),
            None => self.open(window, cx),
        }
    }

    fn choose_highlighted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(listbox) = self.listbox.clone() else {
            // Nothing open, nothing to commit — so this Enter is not ours. The
            // `InputState::copy` shape: an action a component cannot act on is
            // handed outward rather than swallowed.
            cx.propagate();
            return;
        };
        let row = listbox.read(cx).highlighted;
        match row {
            Some(row) => self.choose_row(row, window, cx),
            None => cx.propagate(),
        }
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.listbox.take().is_none() {
            // No popup, so this Escape belongs to whatever encloses us — a
            // `Dialog`, most often. Consuming it unconditionally is how a
            // combobox inside a dialog would stop the dialog closing.
            cx.propagate();
            return;
        }
        cx.notify();
    }
}

/// The wrapper carries the role, not the `TextField` inside it: `TextField`
/// reports no role of its own, and teaching it one here would give a role to
/// every plain text field in the crate — a separate decision, and a wrong one
/// to take by side effect.
///
/// The *value* is the text, not the chosen label. While the two agree that is
/// the same string; while the user is typing, the text is what is on screen and
/// is what a screen reader should read back.
impl<T: Clone + PartialEq + 'static> Accessible for ComboboxState<T> {
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::ComboBox)
            .name(self.label.clone())
            .expanded(self.listbox.is_some());

        if self.disabled {
            a11y.not_focusable("a disabled combobox has nothing for a keyboard to choose between")
        } else {
            // The focus this announces is the text field's, which is a real
            // tab stop of its own. `focusable()` here declares the role's
            // reachability rather than minting a second one.
            a11y.focusable()
        }
    }
}

impl<T: Clone + PartialEq + 'static> Render for ComboboxState<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The *value* is the text, not the chosen label. While the two agree
        // it is the same string; while the user is typing, the text is what is
        // on screen and is what a screen reader should read back. It is added
        // here rather than in `a11y()` because reading it needs an `&App`.
        let a11y = self.a11y().text_value(self.text(cx));
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let full_width = self.full_width;
        let gap = LISTBOX_GAP.to_pixels(window.rem_size());

        let field = text_field(&self.input, cx)
            .control_size(self.size)
            .disabled(self.disabled)
            .full_width(full_width)
            .suffix(Adornment::icon(
                Icons::chevron_down()
                    .size(metrics.text_size)
                    .text_color(theme.fg_muted()),
            ));

        let wrapper = div()
            .id(self.id.clone())
            .announce(a11y)
            .key_context(COMBOBOX_CONTEXT)
            .on_action(cx.listener(|this, _: &ComboboxHighlightNext, window, cx| {
                this.move_highlight(1, window, cx);
            }))
            .on_action(
                cx.listener(|this, _: &ComboboxHighlightPrevious, window, cx| {
                    this.move_highlight(-1, window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &ComboboxChoose, window, cx| {
                this.choose_highlighted(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ComboboxDismiss, window, cx| {
                this.dismiss(window, cx);
            }))
            .relative()
            .when(full_width, |this| this.w_full())
            .child(field);

        #[cfg(test)]
        let wrapper = wrapper.debug_selector(|| "gpuikit-combobox".into());

        wrapper.when_some(self.listbox.clone(), |this, listbox| {
            let popup = div().occlude().child(listbox);

            #[cfg(test)]
            let popup = popup.debug_selector(|| "gpuikit-combobox-popup".into());

            // Rung 1 of `docs/overlays.md`'s ladder, the same one select,
            // popover and context menu use. A chooser's popup is not a new
            // layer.
            this.child(
                deferred(anchored().offset(point(px(0.), gap)).child(popup)).with_priority(1),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, size, Entity, TestAppContext, VisualTestContext};
    use std::cell::RefCell;
    use std::ops::Deref;

    fn options() -> Vec<(usize, &'static str)> {
        vec![(0, "Apple"), (1, "Apricot"), (2, "Banana"), (3, "Cherry")]
    }

    /// `cx.open_window` wants a `Render` root, and an `Entity<ComboboxState>`
    /// is not one — the same wrapper `select.rs`'s tests use.
    struct TestView {
        combobox: Entity<ComboboxState<usize>>,
    }

    impl Render for TestView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(self.combobox.clone())
        }
    }

    /// A live combobox in a real window. `crate::init`, not `theme::init`: the
    /// keyboard model is bindings, and `bind_combobox_keys` is in `init`.
    fn open(
        cx: &mut TestAppContext,
        build: impl FnOnce(Combobox<usize>) -> Combobox<usize>,
    ) -> (Entity<ComboboxState<usize>>, &'static mut VisualTestContext) {
        cx.update(crate::init);
        let window = cx.open_window(size(px(400.), px(300.)), |window, cx| {
            let combobox = cx.new(|cx| {
                ComboboxState::new(build(combobox("test", "Fruit", options())), window, cx)
            });
            TestView { combobox }
        });
        let state = window
            .read_with(cx, |view, _cx| view.combobox.clone())
            .expect("the window's root view is the test view");
        let cx = VisualTestContext::from_window(*window.deref(), cx).into_mut();
        cx.run_until_parked();
        (state, cx)
    }

    fn type_into(state: &Entity<ComboboxState<usize>>, text: &str, cx: &mut VisualTestContext) {
        let text = text.to_string();
        cx.update(|_window, cx| {
            state.update(cx, |this, cx| {
                let input = this.input.clone();
                input.update(cx, |input, cx| input.set_content(text, cx));
            });
        });
        cx.run_until_parked();
    }

    /// Pitfall 5: once a filter is active the popup's row index and the option
    /// index are two different numbers, and committing the wrong one picks the
    /// wrong value in exactly the case no smoke test reaches.
    #[gpui::test]
    fn a_row_index_under_a_filter_is_not_an_option_index(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder);
        type_into(&state, "an", cx);

        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                // "Banana" is option 2 and the only match, so it is row 0.
                assert_eq!(this.visible, vec![2]);
                this.choose_row(0, window, cx);
                assert_eq!(this.selected, Some(2));
                assert_eq!(this.text(cx), SharedString::from("Banana"));
            });
        });
    }

    /// Typing clears the value: a value the text no longer names is a lie.
    #[gpui::test]
    fn typing_clears_the_value(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder.selected(0));
        cx.update(|_window, cx| {
            state.update(cx, |this, cx| {
                assert_eq!(this.selected, Some(0));
                assert_eq!(this.text(cx), SharedString::from("Apple"));
            });
        });

        type_into(&state, "Ap", cx);

        cx.update(|_window, cx| {
            state.update(cx, |this, _cx| {
                assert_eq!(this.selected, None, "typing must clear the value");
                assert!(this.is_open(), "typing opens the popup");
                assert_eq!(this.visible, vec![0, 1]);
            });
        });
    }

    /// Revert puts back the label of the **value**, and typing has already
    /// cleared the value — so a field the user typed nonsense into reverts to
    /// empty, not to what was there before they started. That is the spec's
    /// rule followed to its end rather than an accident: the field shows the
    /// value, there is no value, so it shows nothing.
    #[gpui::test]
    fn blur_reverts_to_the_value_which_typing_cleared(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder.selected(0));
        type_into(&state, "nonsense", cx);
        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                this.blurred(window, cx);
                assert_eq!(this.text(cx), SharedString::from(""));
                assert!(!this.is_open());
            });
        });
    }

    /// Revert with the value still intact: committing a row and then blurring
    /// leaves the committed label alone.
    #[gpui::test]
    fn blur_leaves_a_committed_value_alone(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder.selected(0));
        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                this.choose_row(2, window, cx);
                this.blurred(window, cx);
                assert_eq!(this.text(cx), SharedString::from("Banana"));
                assert_eq!(this.selected, Some(2));
            });
        });
    }

    #[gpui::test]
    fn blur_reverts_to_empty_with_no_value(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder);
        type_into(&state, "nonsense", cx);
        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                this.blurred(window, cx);
                assert_eq!(this.text(cx), SharedString::from(""));
            });
        });
    }

    #[gpui::test]
    fn keep_leaves_unmatched_text_alone(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder.keep_unmatched_text());
        type_into(&state, "nonsense", cx);
        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                this.blurred(window, cx);
                assert_eq!(this.text(cx), SharedString::from("nonsense"));
                assert_eq!(this.selected, None);
            });
        });
    }

    #[gpui::test]
    fn create_hands_the_text_to_the_handler(cx: &mut TestAppContext) {
        let seen = Rc::new(RefCell::new(Vec::<SharedString>::new()));
        let recorder = seen.clone();
        let (state, cx) = open(cx, move |builder| {
            builder.on_create(move |text, _window, _cx| recorder.borrow_mut().push(text))
        });
        type_into(&state, "Durian", cx);
        cx.update(|window, cx| {
            state.update(cx, |this, cx| {
                assert_eq!(this.unmatched, UnmatchedText::Create);
                this.blurred(window, cx);
            });
        });
        assert_eq!(seen.borrow().as_slice(), &[SharedString::from("Durian")]);
    }

    /// The default filter, on its own — the one piece of matching this crate
    /// does own.
    #[gpui::test]
    fn the_default_filter_is_a_case_insensitive_substring(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder);
        cx.update(|_window, cx| {
            state.update(cx, |this, _cx| {
                assert_eq!(this.refilter("AP").len(), 2);
                assert_eq!(this.refilter("rr").len(), 1);
                assert_eq!(this.refilter("").len(), 4);
            });
        });
    }

    /// The claim this element declines, read back from where it is declared.
    /// A row of a combobox popup must not claim `active_descendant`: the popup
    /// is a sibling of the focused field, gpui honours the property only under
    /// a focused ancestor, and the claim would be dropped in silence.
    #[gpui::test]
    fn no_row_claims_active_descendant(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, |builder| builder);
        type_into(&state, "a", cx);
        cx.update(|_window, cx| {
            state.update(cx, |this, cx| {
                let listbox = this.listbox.clone().expect("typing opened the popup");
                let listbox = listbox.read(cx);
                assert!(listbox.highlighted.is_some(), "a row is highlighted");
                for index in 0..listbox.options.len() {
                    assert!(
                        !listbox.row_a11y(index).is_active_descendant(),
                        "row {index} claims an active descendant a focused ancestor \
                         could never honour"
                    );
                }
            });
        });
    }
}
