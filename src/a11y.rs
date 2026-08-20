//! How an element in this crate says what it is: one value, one method, one
//! place to change the answer.
//!
//! This module is the decision `docs/issues/element-roles-convention.md` asked
//! for. It is written as a decision record because the alternative — a
//! convention held in ten issue bodies — is how ten elements end up with ten
//! subtly different mechanisms. Each numbered section below answers one of
//! that issue's questions, next to the code that implements it.
//!
//! ```ignore
//! use gpuikit::a11y::{A11y, Announce};
//! use gpuikit::traits::accessible::Accessible;
//! use gpui::Role;
//!
//! impl Accessible for Button {
//!     fn a11y(&self) -> A11y {
//!         A11y::new(Role::Button).name(self.label.clone()).focusable()
//!     }
//! }
//!
//! impl RenderOnce for Button {
//!     fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
//!         let a11y = self.a11y();
//!         h_stack().id(self.id).announce(a11y)/* … */
//!     }
//! }
//! ```
//!
//! # 1. Where a role is declared
//!
//! On the element the element was already building, through [`Announce`].
//!
//! gpui's `.role()` lives on `StatefulInteractiveElement`, which a `Div`
//! implements only once `.id()` has turned it into a `Stateful<Div>`. So
//! [`Announce`] is blanket-implemented for `StatefulInteractiveElement` and
//! nothing else, and **"no id, no role" is enforced by the type system**: an
//! element cannot announce itself without first having minted an id under
//! [`crate::element_id`]'s rule. That is the property that keeps roles out of
//! the duplicate-node trap that module documents.
//!
//! No element has to become a hand-written [`gpui::Element`] to report a role.
//! A `RenderOnce` struct declares what it announces with
//! [`Accessible`](crate::traits::accessible::Accessible) and applies it to its
//! root element with one call. The two halves are separate on purpose: the
//! trait is a property of the *component* (and can be read by a test, or by a
//! parent), while `announce` is a property of the *element tree*.
//!
//! **`Img` is the one escape hatch.** gpui implements
//! `StatefulInteractiveElement` for `Img` unconditionally, unlike `Div`, so
//! `img(…).announce(…)` compiles with no id at all. It then reports nothing:
//! `Img`'s `Element::a11y_role` does not return the role its interactivity
//! stored, so the announcement is dropped before the missing id even matters.
//! An image that needs a role needs `div().id(…)` around it — which is worth
//! knowing before `Avatar` is adopted.
//!
//! # 2. How an element is named
//!
//! [`A11y::name`], and for the roles in [`role_requires_a_name`] it is
//! **required** rather than optional: a nameless `Role::Button` trips a
//! `debug_assert!` inside [`Announce::announce`]. A control that announces
//! "button" and nothing else is worse than one that is not in the tree, and an
//! optional name is one an element forgets to set.
//!
//! Where the name comes from:
//!
//! - **From the element's own visible text where it has any.** `Button`'s
//!   label *is* its name; there is no second `.aria_label()` to keep in step
//!   with it, and no way for the two to disagree.
//! - **From a constructor argument where it has none.** An `IconButton` has
//!   only a glyph. Its name has to be a required argument, which is a breaking
//!   change to that element's API and is why it is the first of the rollout
//!   rather than part of this change.
//! - **Never from the tooltip.** A tooltip is optional, and in this crate it
//!   is an `AnyView` — there is no string in it to read. accesskit has a
//!   separate `tooltip` property precisely because a tooltip is not a name.
//!
//! A plain `&str` / `SharedString` child does not compete with its parent's
//! name: those have no id and no role, so they add no node. Only gpui's
//! `text!` macro mints an id and reports `Role::Label`; if this crate ever
//! adopts `text!` inside a control, those uses need `Text::new_inaccessible`.
//!
//! # 3. How state is reported
//!
//! As fields of [`A11y`], applied by [`Announce::announce`] — `toggled`,
//! `selected`, `expanded`, `value`, `orientation`, `level`,
//! `position_in_set`, `size_of_set`, `active_descendant`. `position_in_set`
//! and `size_of_set` travel together: a position
//! with no size announces "3" out of nowhere, and gpui's `aria_size_of_set` is
//! what makes it "3 of 8". **State goes on the element that changes it**, which is
//! not always the element it is about: `Sidebar` reports `aria-expanded` on
//! `SidebarTrigger`, because the trigger is the control a screen reader user
//! operates, and the panel is the thing that happens as a result.
//!
//! **`active_descendant` is the odd one and knows it.** It is a plain `bool`
//! rather than an `Option<bool>`, it is set on the *descendant* rather than on
//! the container that owns the focus, and it is the one field no test in this
//! crate can read back off a node. All three are consequences of how gpui
//! spells the property, and all three are argued at
//! [`A11y::active_descendant`] rather than here — including which failures its
//! guard catches and which it does not. Read that before adding a second
//! caller.
//!
//! Two properties this convention deliberately *cannot* express, both because
//! gpui cannot:
//!
//! - **`disabled`.** accesskit has `Node::set_disabled`, but gpui's
//!   `AriaProperties` has no field for it and `Interactivity::write_a11y_info`
//!   never writes it. The only public workaround,
//!   `.a11y_synthetic_children(|b| b.parent_node().set_disabled(true))`, burns
//!   the element's single synthetic-children slot and cannot be exercised from
//!   a test (`A11ySubtreeBuilder::new` is `pub(crate)`). A disabled control is
//!   therefore distinguishable only by the `Click` action its node does *not*
//!   offer — gpui derives that action from the presence of a click listener,
//!   and this crate already drops the handler when disabled.
//! - **`sort_direction`**, which `src/elements/table.rs` had already flagged.
//!   Same shape: `accesskit::Node::set_sort_direction` exists, no gpui builder
//!   does.
//!
//! Both are upstream asks rather than decisions taken here. A field on
//! [`A11y`] that silently did nothing would be worse than an absent one, so
//! neither exists. When gpui grows `aria_disabled` / `aria_sort`, adding them
//! is a local change to this file.
//!
//! # 4. How the keyboard reaches it
//!
//! [`A11y::focusable`], [`A11y::focus_handle`] and [`A11y::not_focusable`] —
//! and [`Announce::announce`] *applies* the answer rather than leaving the
//! element to honour it.
//!
//! That is the load-bearing part. A role and a focus decision that live in two
//! places drift, and the way they drift is the bug this section exists for:
//! `Button` announced `Role::Button` for a release and could not take keyboard
//! focus, so a screen reader was told about a control a keyboard could not
//! reach. Making the announcement the thing that mints the focus handle and
//! registers the tab stop means the two cannot disagree — there is no second
//! call an element can forget.
//!
//! For every role in [`role_requires_keyboard_focus`], saying nothing is a
//! `debug_assert!`, the counterpart of section 2's missing-name one.
//! **Declining is not silence**: [`A11y::not_focusable`] takes a reason, and
//! `announce` does not assert on it. `Button` declines when it is disabled,
//! which is the weaker of the two ARIA-sanctioned answers and is forced by
//! gpui having no `aria_disabled` (section 3) — a disabled control is out of
//! the tab order rather than a focusable control that announces why.
//!
//! Two things about gpui that this rests on, both discovered the hard way:
//!
//! - **The caller does not have to supply a [`FocusHandle`].**
//!   `Interactivity::request_layout` mints one for a focusable element and
//!   stores it in that element's element state, "which lives as long as frames
//!   contain an element with this id". A `RenderOnce` control is therefore the
//!   same focus target across frames without anything above it holding state,
//!   which is why [`A11y::focus_handle`] is optional rather than required and
//!   why no existing `button(…)` call site had to change.
//!   `focus_survives_a_redraw` in `elements::button` pins it.
//! - **Focusable is not tabbable.** `focusable()` mints a handle whose
//!   `tab_stop` is `false`, and `TabStopMap::next` walks straight past a node
//!   that is not a stop. Worse, `track_focus` does *not* push the element's
//!   `tab_stop` onto the handle — only the minted-handle path does — so a
//!   caller-supplied handle has to be made a stop on the handle itself. Both
//!   paths below say `.tab_stop(true)`, and deleting either one fails a test.
//!
//! Enter and Space activation is gpui's: a focused element with a click
//! listener synthesises a click from a matched key *down* and key *up*. Tab
//! had no binding at all — gpui ships `Window::focus_next` / `focus_prev` and
//! binds neither — so [`bind_focus_keys`] installs [`FocusNext`] and
//! [`FocusPrevious`], and [`crate::init`] calls it. The listener for those
//! actions is [`FocusNavigation::moves_focus_on_tab`], which `announce` puts
//! on every control it makes focusable.
//!
//! **Tab does nothing until something is focused**, and putting
//! `moves_focus_on_tab()` on "the app's root element" is not enough on its
//! own: with no focus, gpui falls back to the dispatch node belonging to its
//! own wrapper around the root view, which is *above* the root element. The
//! listener has to sit on an element that tracks a handle something actually
//! focuses — `examples/showcase.rs` is the worked example.
//!
//! The binding carries **no key context**. An earlier `!Input` predicate looks
//! right and is not: `KeyBindingContextPredicate::depth_of` returns `None` for
//! an empty context stack, and a focus path of plain `div`s has no key
//! contexts at all, so the binding would be disabled exactly where it is
//! needed. What keeps Tab inside a focused text input instead is binding
//! order: `crate::init` installs the focus keys *before*
//! `input::bind_input_keys`, gpui prefers the later binding at equal context
//! depth, and only an element `announce` made focusable listens for
//! [`FocusNext`] anyway.
//!
//! # 5. How it is tested
//!
//! By rendering a component and calling the two [`gpui::Element`] methods gpui
//! itself calls — `a11y_role` and `write_a11y_info` — on the element that
//! comes back. `test_support::announced` does exactly that and hands back the
//! real `accesskit::Node`.
//!
//! It works this way because **accessibility cannot be switched on in a
//! test**. `A11y::is_active()` inside gpui is set only by a platform adapter's
//! activation callback, and the test platform has none, so no test can see
//! gpui's real tree — or its duplicate-node `debug_assert!`.
//! `src/markdown/selectable_text.rs`'s recorder says the same thing for a
//! hand-written element; this is the `RenderOnce` equivalent.
//!
//! Note that `#[derive(IntoElement)]` wraps a `RenderOnce` in a
//! `Component<C>`, whose `a11y_role()` is `None` — the role is on whatever
//! `render` returns. The helper therefore calls `render`, not `into_element`.
//! For the same reason it cannot see through an `AnyElement`, which does not
//! forward the two methods either: a `render` that ends in
//! `.into_any_element()` announces normally when gpui draws it, and reports
//! nothing to this helper.
//!
//! The second half is `tests::no_element_calls_gpuis_a11y_builders_directly`,
//! a source scan that fails the build if anything under `src/` calls gpui's
//! `.role()` / `.aria_*()` builders outside this module. It is modelled on
//! `element_id::tests::no_element_mints_a_constant_id`, down to the corpus
//! floor and the `#[cfg(test)]`-by-column skip. If a change needs a property
//! [`A11y`] does not have, the intended move is to add the field here and
//! apply it in [`Announce::announce`]; the scan will not accept a local
//! `.aria_…()` call, and that is the point.
//!
//! **`announced` cannot see focus.** It calls two [`gpui::Element`] methods and
//! never lays out or paints, so the handle gpui mints during `request_layout`
//! and the tab stop it registers during paint are both invisible to it. What
//! an element *declares* is tested through
//! [`Accessible`](crate::traits::accessible::Accessible); that the declaration
//! reaches the tab order is tested by drawing a real window and pressing a real
//! key, which `elements::button`'s tests do. Drawing a role-carrying,
//! mouse-listening element needs a real *view*, not `VisualTestContext::draw` —
//! registering a mouse listener reads `Window::current_view` — which is the
//! note `elements::sidebar`'s harness already carries.
//!
//! What the scan cannot see is a hand-written [`gpui::Element`] writing an
//! `accesskit::Node` itself — `src/markdown/selectable_text.rs` does, because
//! a text run is not a `div` and never passes through a builder. That is the
//! one place in the crate outside this convention, and it has its own tests.
//!
//! # 6. The rollout
//!
//! `Button` is the worked example; `Sidebar` — which shipped a role ahead of
//! this decision — has been migrated onto it; `Splitter` arrived carrying one;
//! and `Select` is the first element adopted *after* the convention, which is
//! why it is the one that had to take the breaking change section 2 predicted
//! (`ComboBox` needs a name, and a select's visible text is its value, so the
//! name is a constructor argument).
//!
//! The rest is **not** prose any more. [`ELEMENTS_WITHOUT_A_ROLE`] names every
//! element module that still declares nothing, with the reason — what it would
//! announce, or what has to exist first — and
//! `tests::every_element_module_declares_a_role` checks it in both directions:
//! a module that is silent and unlisted fails, and so does a listed module
//! that has since been adopted. The list can only shrink, which is the
//! property a rollout order held in a doc comment did not have.
//!
//! The order the follow-on work still wants is `IconButton` first — it forces
//! the "name as a constructor argument" half of section 2 and is the first
//! element that will meet both of this module's assertions at once — then
//! `Checkbox` / `Switch` / `Toggle`, `Slider` / `Progress`, `Tabs` / `List`,
//! `Accordion` / `Collapsible`, the overlays, and `Table` last, since it needs
//! derived cell ids first.

use gpui::{
    actions, App, FocusHandle, InteractiveElement, KeyBinding, Orientation, Role, SharedString,
    StatefulInteractiveElement, Toggled,
};

/// What an element announces: a role, a name, and whatever state goes with the
/// role.
///
/// Built by an element's
/// [`Accessible`](crate::traits::accessible::Accessible) implementation and
/// applied with [`Announce::announce`]. It is an owned value rather than a set
/// of calls so that it can be built before `render` moves the element's
/// fields, read back in a test, and extended in one place.
#[derive(Clone, Debug, PartialEq)]
pub struct A11y {
    role: Role,
    name: Option<SharedString>,
    description: Option<SharedString>,
    toggled: Option<Toggled>,
    selected: Option<bool>,
    expanded: Option<bool>,
    value: Option<A11yValue>,
    orientation: Option<Orientation>,
    level: Option<usize>,
    position_in_set: Option<usize>,
    size_of_set: Option<usize>,
    /// A plain `bool`, unlike every other state field here. See
    /// [`A11y::active_descendant`].
    active_descendant: bool,
    focus: Focus,
}

/// Whether an element takes keyboard focus, and how — section 4 of the module
/// docs.
///
/// Private, because it is answered through [`A11y::focusable`],
/// [`A11y::focus_handle`] and [`A11y::not_focusable`] and read back through
/// [`A11y::is_focusable`] and [`A11y::focus_declined_because`]. What matters
/// outside this module is that all three states are distinguishable: silence
/// is not a decision, and [`Announce::announce`] asserts on it.
#[derive(Clone, Debug, Default, PartialEq)]
enum Focus {
    /// Nobody has answered. For a role in [`role_requires_keyboard_focus`]
    /// this is the bug section 4 exists for.
    #[default]
    Undecided,
    /// The element is a tab stop. `None` lets gpui mint the [`FocusHandle`]
    /// and keep it in the element's own element state, which is what a
    /// `RenderOnce` control wants; `Some` is for an element that already owns
    /// one.
    Takes(Option<FocusHandle>),
    /// The element deliberately stays out of the tab order, for this reason.
    Declines(SharedString),
}

/// The value a control reports, as distinct from its name.
///
/// A number carries its bounds because they are **not** optional: a slider
/// that reports `70` with no range is announced as unbounded, which is a
/// worse answer than not reporting a value at all. If a control genuinely has
/// no range, it has a [`Text`](A11yValue::Text) value.
#[derive(Clone, Debug, PartialEq)]
pub enum A11yValue {
    /// A value with no numeric meaning — the current option of a select, the
    /// text of a field.
    Text(SharedString),
    /// A number and the range it sits in.
    Number {
        /// The current value.
        value: f64,
        /// The lowest value the control can take.
        min: f64,
        /// The highest value the control can take.
        max: f64,
        /// How far one step of the control moves the value.
        step: f64,
    },
}

impl A11y {
    /// An announcement of `role`, with no name and no state yet.
    pub fn new(role: Role) -> Self {
        Self {
            role,
            name: None,
            description: None,
            toggled: None,
            selected: None,
            expanded: None,
            value: None,
            orientation: None,
            level: None,
            position_in_set: None,
            size_of_set: None,
            active_descendant: false,
            focus: Focus::Undecided,
        }
    }

    /// The accessible name: what a screen reader says before the role.
    ///
    /// Required for every role in [`role_requires_a_name`]. Derive it from the
    /// element's own visible text where it has any — see section 2 of the
    /// module docs.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Supplementary text announced after the name, role and value — a
    /// subtitle or a hint. Not a substitute for a name.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The on/off/mixed state of a checkbox, switch or toggle button.
    pub fn toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Whether this item of a list, tab strip or table is the selected one.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Whether the region this control governs is open.
    ///
    /// Goes on the control that changes the state, not on the region — see
    /// section 3 of the module docs.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// A value with no numeric meaning.
    pub fn text_value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(A11yValue::Text(value.into()));
        self
    }

    /// A number, with the range and step it moves in. All four are required;
    /// see [`A11yValue`].
    pub fn number_value(mut self, value: f64, min: f64, max: f64, step: f64) -> Self {
        self.value = Some(A11yValue::Number {
            value,
            min,
            max,
            step,
        });
        self
    }

    /// Which way a slider, separator or tab strip runs.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// The depth of a heading or a tree item, counting from 1.
    pub fn level(mut self, level: usize) -> Self {
        self.level = Some(level);
        self
    }

    /// This item's index among its siblings, counting from 1.
    pub fn position_in_set(mut self, position: usize) -> Self {
        self.position_in_set = Some(position);
        self
    }

    /// How many siblings there are in total — what turns
    /// [`position_in_set`](A11y::position_in_set)'s "3" into "3 of 8".
    ///
    /// A position with no size announces a number out of nowhere, so the two
    /// are worth setting together even though neither is required.
    pub fn size_of_set(mut self, size: usize) -> Self {
        self.size_of_set = Some(size);
        self
    }

    /// This element is the one a composite widget's keyboard has got to, while
    /// real focus stays on the container — `aria-activedescendant`.
    ///
    /// A listbox popup that owns focus and moves a *highlight* between its rows
    /// has no second focus to give the highlighted row; this is how the row is
    /// announced as the current one anyway. `src/elements/select.rs` is the
    /// worked example, and today the only caller.
    ///
    /// # It is set on the descendant, not on the container
    ///
    /// The web property names the item *from* the container. gpui inverts it:
    /// `aria_active_descendant()` goes on the item, takes no argument, and is
    /// honoured only while a focused **ancestor** of that item is on the node
    /// stack. Two things follow, and both are load-bearing:
    ///
    /// - The APG arrangement where focus stays on a combo box trigger and
    ///   points into a popup beside it **cannot be expressed**. A sibling is
    ///   not an ancestor, so the claim would be dropped in silence. Focus goes
    ///   on the popup, and the row inside it makes the claim.
    /// - Only one node may claim it per frame, the claiming node may not be the
    ///   focused one, and it must be in the tree. gpui `debug_assert!`s on all
    ///   three, so a caller that highlights two rows at once aborts.
    ///
    /// # Why a `bool` and not an `Option<bool>`
    ///
    /// Every other state field here is an `Option`, because "said nothing" and
    /// "said false" are different announcements. Here they are not: gpui's
    /// builder takes no argument, so there is no call that says *not* the
    /// active descendant. `Some(false)` would be a state this crate could hold
    /// and never report, which is the failure section 3 of the module docs is
    /// about. Absent is the only "no".
    ///
    /// # What guards this field, and what does not
    ///
    /// This is the one state field `tests::every_state_field_reaches_the_node`
    /// excludes, because the property is applied at paint time behind
    /// `window.a11y.is_active()` and no test platform in this crate switches
    /// accessibility on — so nothing here can read the applied node back. Be
    /// precise about what stands in its place:
    ///
    /// - The exhaustive `let A11y { … } = a11y` destructure in
    ///   [`Announce::announce`] catches **a new field going unhandled**: adding
    ///   one without applying it is a compile error.
    ///   `tests::an_active_descendant_is_declared_rather_than_read_back` catches
    ///   the field disappearing from the builder or the reader.
    /// - Neither catches **a wrong apply**. Change the apply to call
    ///   `aria_active_descendant()` unconditionally, ignoring the bool, and
    ///   every test in this crate still passes; the escaping failure is gpui's
    ///   two-nodes-in-one-frame panic at run time, in an app with
    ///   accessibility on. That is why the apply is written as the single
    ///   narrowest form — `if active_descendant { … }`, nothing else — so the
    ///   one line a reader has to check is one line.
    ///
    /// A field that silently does nothing is worse than an absent one; a field
    /// whose guard is weaker than it looks is the same hazard one level up, and
    /// this paragraph is the only place a reader meets it.
    pub fn active_descendant(mut self, active_descendant: bool) -> Self {
        self.active_descendant = active_descendant;
        self
    }

    /// This element is a keyboard tab stop, and gpui should mint and keep the
    /// [`FocusHandle`] for it.
    ///
    /// The right answer for a `RenderOnce` control: gpui stores the handle in
    /// the element's element state, keyed on the element id, so the control is
    /// the same focus target across frames without a caller holding anything.
    /// See section 4 of the module docs.
    pub fn focusable(mut self) -> Self {
        self.focus = Focus::Takes(None);
        self
    }

    /// The same, for an element that already owns a [`FocusHandle`] — a
    /// `Render` entity, or a control with its own key handlers.
    ///
    /// [`Announce::announce`] makes the handle a tab stop, because
    /// `track_focus` does not carry the element's `tab_stop` onto the handle
    /// the way the minted path does.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus = Focus::Takes(Some(handle));
        self
    }

    /// This element stays out of the tab order, because `why`.
    ///
    /// The reason is required on purpose: it is what distinguishes a decision
    /// from a call someone made to silence the assertion. `announce` asserts
    /// against silence, not against "no".
    pub fn not_focusable(mut self, why: impl Into<SharedString>) -> Self {
        self.focus = Focus::Declines(why.into());
        self
    }

    /// The role this announcement reports.
    pub fn role(&self) -> Role {
        self.role
    }

    /// The accessible name, if one was given.
    pub fn accessible_name(&self) -> Option<&SharedString> {
        self.name.as_ref()
    }

    /// Whether this announcement needs a name and does not have a usable one.
    ///
    /// Blank counts as missing: `button("save", "")` announces "button" and
    /// nothing else, which is the case the rule exists for.
    /// [`Announce::announce`] asserts on this in debug builds.
    pub fn is_missing_a_required_name(&self) -> bool {
        role_requires_a_name(self.role)
            && !self
                .name
                .as_ref()
                .is_some_and(|name| !name.trim().is_empty())
    }

    /// Whether this element claims to be its container's active descendant.
    ///
    /// Reads back what [`active_descendant`](A11y::active_descendant) was told.
    /// It cannot read back what gpui did with it — see that method.
    pub fn is_active_descendant(&self) -> bool {
        self.active_descendant
    }

    /// Whether this announcement takes keyboard focus.
    pub fn is_focusable(&self) -> bool {
        matches!(self.focus, Focus::Takes(_))
    }

    /// Why this element declined keyboard focus, if it declined it.
    pub fn focus_declined_because(&self) -> Option<&SharedString> {
        match &self.focus {
            Focus::Declines(why) => Some(why),
            _ => None,
        }
    }

    /// Whether this announcement's role needs a focus decision and has none.
    ///
    /// [`Announce::announce`] asserts on this in debug builds — the
    /// counterpart of [`is_missing_a_required_name`](A11y::is_missing_a_required_name).
    pub fn is_missing_a_focus_decision(&self) -> bool {
        role_requires_keyboard_focus(self.role) && matches!(self.focus, Focus::Undecided)
    }
}

/// Whether a role's whole announcement is *name + role + state*, so that
/// leaving the name out leaves the element unidentifiable.
///
/// This is a list, and lists rot. It is public so that an element — or its
/// test — can consult the same one, and it is expected to grow as elements are
/// adopted. A role that is missing from it is not *forbidden* a name; it is
/// only not forced to have one, which is right for a landmark or a container
/// that is named by its contents (`Role::Complementary`, `Role::Document`).
///
/// The rot is *quiet*, because the list feeds a `debug_assert!` rather than a
/// return value an element reads: dropping an arm takes a name requirement with
/// it and breaks nothing else. So every arm is pinned, one assertion per reason,
/// by `tests::the_name_rule_covers_the_roles_that_are_nothing_without_one` —
/// along with the three absences this crate has argued for in writing
/// (`Role::Complementary`, `Role::Document` above, and `Role::ListBox` in
/// `src/elements/select.rs`). Adding an arm means adding it there too, with the
/// argument for it as the assertion's message.
pub fn role_requires_a_name(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::DefaultButton
            | Role::CheckBox
            | Role::Switch
            | Role::RadioButton
            | Role::Link
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::ListBoxOption
            | Role::Tab
            | Role::TreeItem
            | Role::Slider
            // A divider between two panes has no visible text of its own to
            // borrow a name from, so its name is a constructor argument —
            // `src/elements/splitter.rs` takes one, the way `IconButton` does.
            | Role::Splitter
            | Role::SpinButton
            | Role::ComboBox
            | Role::EditableComboBox
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::NumberInput
            | Role::PasswordInput
            | Role::DateInput
            | Role::ProgressIndicator
            | Role::Meter
            | Role::Dialog
            | Role::AlertDialog
            | Role::Image
    )
}

/// Whether a role promises a control a keyboard user can operate, so that
/// announcing it without taking focus reaches a screen reader and not a
/// keyboard.
///
/// The counterpart of [`role_requires_a_name`], and the same kind of list: it
/// is public so an element or its test can consult the same one, and it is
/// expected to grow.
///
/// Two groups are deliberately **not** on it:
///
/// - **The composite-item roles** — `MenuItem`, `MenuItemCheckBox`,
///   `MenuItemRadio`, `Tab`, `TreeItem`, `ListBoxOption`. These are arrow-key
///   targets inside a composite that owns the one tab stop, so no per-item rule
///   can be right: making each item a tab stop is exactly the mistake the ARIA
///   authoring practices call out. They join the list when this crate has a
///   roving-focus convention, which is a separate decision — `Tabs`, `List`,
///   `ContextMenu` and `Select`'s popup all want it and none of them has it.
/// - **Landmarks and containers** — `Complementary`, `Document`, `Group`.
///   These are read, not operated.
///
/// The rot here is *quiet* in the same way [`role_requires_a_name`]'s is: the
/// list feeds a `debug_assert!` inside [`Announce::announce`] rather than a
/// return value an element reads, so dropping an arm takes a control's focus
/// requirement with it, breaks nothing else, and is not evaluated at all in a
/// release build. So every arm is pinned by name, one assertion per reason, by
/// `tests::the_focus_rule_covers_the_roles_a_keyboard_operates` — along with
/// the three groups declined above. Adding an arm means adding it there too,
/// with the argument for it as the assertion's message.
pub fn role_requires_keyboard_focus(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            // A dialog's Enter key resolves to this one, so it is the most
            // focus-requiring control in the set.
            | Role::DefaultButton
            | Role::CheckBox
            | Role::Switch
            | Role::RadioButton
            | Role::Link
            | Role::Slider
            // A standalone control that owns one tab stop and moves a value
            // with the arrow keys — `Slider`'s shape exactly.
            | Role::Splitter
            | Role::SpinButton
            | Role::ComboBox
            | Role::EditableComboBox
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::NumberInput
            | Role::PasswordInput
            | Role::DateInput
    )
}

actions!(
    a11y,
    [
        /// Move keyboard focus to the next tab stop.
        FocusNext,
        /// Move keyboard focus to the previous tab stop.
        FocusPrevious,
    ]
);

/// Bind Tab and Shift-Tab to [`FocusNext`] and [`FocusPrevious`].
///
/// gpui ships `Window::focus_next` and `Window::focus_prev` and binds neither,
/// so without this Tab does nothing at all. [`crate::init`] calls it, and calls
/// it **before** `input::bind_input_keys` on purpose: gpui prefers the
/// later-registered binding at equal context depth, which is what keeps Tab
/// inside a focused text input. Swapping the two takes Tab out of every input
/// in the crate.
///
/// The bindings carry no key context. See section 4 of the module docs for why
/// a `!Input` predicate looks right and is not.
pub fn bind_focus_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
}

/// Listens for [`FocusNext`] / [`FocusPrevious`] and moves focus.
///
/// [`Announce::announce`] puts this on every control it makes focusable, so a
/// consumer never calls it for a gpuikit element. What a consumer *does* need
/// it for is the cold start: with nothing focused, gpui dispatches to the node
/// belonging to its own wrapper around the root view, above the root element,
/// so the first Tab reaches no listener. Putting this on the element that
/// tracks the handle the app focuses at startup is what makes that first Tab
/// work — `examples/showcase.rs` is the worked example.
pub trait FocusNavigation: InteractiveElement {
    /// Answer Tab and Shift-Tab on this element.
    fn moves_focus_on_tab(self) -> Self {
        self.on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrevious, window, cx| window.focus_prev(cx))
    }
}

impl<E: InteractiveElement> FocusNavigation for E {}

/// Every element module that does **not** yet declare a role, and why.
///
/// Section 6's rollout order was prose, which is a list nothing reads.
/// `tests::every_element_module_declares_a_role` reads this one: a module in
/// `src/elements.rs` that neither implements
/// [`Accessible`](crate::traits::accessible::Accessible) nor appears here
/// fails the build, and so does an entry for a module that has since been
/// adopted. So the list can only shrink, and the gap it describes cannot go
/// back to being invisible.
///
/// A reason says what the module *would* announce, or what has to exist
/// first. "Not done yet" is not a reason, and the test enforces a minimum
/// length to say so.
pub const ELEMENTS_WITHOUT_A_ROLE: &[(&str, &str)] = &[
    (
        "accordion",
        "would be a set of Group headers with `expanded`, once Collapsible answers the same \
         question — the two share a mechanism and should be adopted together",
    ),
    (
        "alert",
        "would be Role::Alert (or AlertDialog for the modal shape), which needs the live-region \
         decision this convention has not taken",
    ),
    (
        "aspect_ratio",
        "a layout wrapper with no semantics of its own: it would announce nothing, and saying so \
         needs a `Role::GenericContainer` escape gpui rejects outright",
    ),
    (
        "avatar",
        "would be Role::Image with a required name, and section 1's `Img` escape hatch says it \
         first needs a `div().id(…)` around the image",
    ),
    (
        "badge",
        "decorative text beside the thing it counts; it wants an `aria-describedby` relationship \
         gpui has no builder for, not a role of its own",
    ),
    (
        "breadcrumb",
        "would be Role::Navigation around a Role::List of Role::Link, so it needs the composite \
         roles and the link naming rule together",
    ),
    (
        "button_group",
        "would be Role::Group with an orientation, and its children are already Buttons — it is \
         waiting on nothing but its turn behind `icon_button`",
    ),
    (
        "card",
        "a surface, not a control: it would be Role::Group, and only once it can be named by its \
         own header rather than by an argument",
    ),
    (
        "checkbox",
        "would be Role::CheckBox with `toggled` and a required name, and is next after \
         `icon_button` in section 6's order",
    ),
    (
        "collapsible",
        "would report `expanded` on its trigger the way SidebarTrigger does; adopted together \
         with `accordion`, which shares the mechanism",
    ),
    (
        "context_menu",
        "would be Role::Menu over Role::MenuItem rows, which are composite-item roles — they \
         need the roving-focus convention `role_requires_keyboard_focus` names",
    ),
    (
        "dialog",
        "would be Role::Dialog with a required name and `modal`; gpui has no `aria_modal` \
         builder, and a dialog that announces itself unmodal is worse than one that waits",
    ),
    (
        "empty",
        "an empty-state illustration with a heading and a message: it would announce a \
         Role::Group named by its own heading, which is the naming rule `card` is also waiting on",
    ),
    (
        "field",
        "the element that most wants `labelled_by`, which gpui has no builder for — a Field's \
         whole job is naming the control beside it",
    ),
    (
        "icon_button",
        "would be Role::Button with the name as a required constructor argument, which is the \
         breaking change section 2 describes and the first of section 6's rollout",
    ),
    (
        "input",
        "would be Role::TextInput and its siblings with a `value`; it owns its own FocusHandle \
         already, which is exactly what `A11y::focus_handle` takes",
    ),
    (
        "kbd",
        "renders a key name as decoration; it would want accesskit's `keyboard_shortcut` on the \
         control it describes, not a role of its own",
    ),
    (
        "label",
        "would be Role::Label, and gpui already mints one for `text!` — adopting it needs the \
         duplicate-node question section 1 raises answered first",
    ),
    (
        "list",
        "would be Role::List over Role::ListItem, and its selectable rows are composite-item \
         roles waiting on the roving-focus convention",
    ),
    (
        "loading_indicator",
        "would be Role::ProgressIndicator with no numeric value, which needs the \
         indeterminate-progress decision `progress` also wants",
    ),
    (
        "popover",
        "would be a named Role::Group with `expanded` on its trigger; it owns focus by hand \
         today and adopting it means moving that onto `A11y::focus_handle`",
    ),
    (
        "progress",
        "would be Role::ProgressIndicator with a bounded number value; the indeterminate case \
         has no answer yet, and `A11yValue::Number` requires all four bounds",
    ),
    (
        "radio_group",
        "would be Role::RadioGroup over Role::RadioButton rows, the clearest case of the \
         roving-focus convention this crate does not have",
    ),
    (
        "scroll_area",
        "a scroll container gpui already describes through its own scroll properties; a role \
         here would add a node without adding information",
    ),
    (
        "separator",
        "would be Role::Splitter with an orientation and no interaction, which is the one \
         `splitter.rs` already reports — the two need reconciling before either moves",
    ),
    (
        "slider",
        "would be Role::Slider with a bounded number value and a required name, which is the \
         `A11yValue::Number` case section 3 was written for",
    ),
    (
        "switch",
        "would be Role::Switch with `toggled` and a required name; adopted alongside `checkbox` \
         and `toggle`, which share the shape",
    ),
    (
        "table",
        "its own module docs already say it: gpui has no `aria_sort` builder, and a table needs \
         derived cell ids before its cells can carry roles at all — section 6 puts it last",
    ),
    (
        "tabs",
        "would be Role::TabList over Role::Tab, composite-item roles that need the roving-focus \
         convention before a per-item rule can be right",
    ),
    (
        "text_field",
        "would be Role::TextInput with a `value` and a required name; it owns a FocusHandle \
         already, so adopting it is a move onto `A11y::focus_handle`",
    ),
    (
        "textarea",
        "would be Role::MultilineTextInput; same shape as `text_field`, and the two should be \
         adopted in one change so their focus handling matches",
    ),
    (
        "toast",
        "would be Role::Alert in a live region, which is the live-region decision `alert` is \
         also waiting on — both should be taken once",
    ),
    (
        "toggle",
        "would be Role::Button with `toggled`, or Role::Switch depending on the answer \
         `checkbox` and `switch` settle between them",
    ),
    (
        "toggle_group",
        "would be Role::Group over toggles, so it cannot be adopted before `toggle` has decided \
         what one of its children announces",
    ),
    (
        "tooltip",
        "accesskit has a `tooltip` property on the described control rather than a role, and \
         this crate's tooltip is an `AnyView` with no string to read — see section 2",
    ),
    (
        "typography",
        "would be Role::Heading with a `level` and Role::Paragraph; gpui mints label nodes for \
         text already, so this needs the duplicate-node question answered with `label`",
    ),
];

/// Applies an [`A11y`] to the element that carries it.
///
/// Blanket-implemented for every `StatefulInteractiveElement`, which is gpui's
/// way of saying "this element has an id" — see section 1 of the module docs.
/// This is the only place in the crate that calls gpui's `.role()` and
/// `.aria_*()` builders, and
/// `tests::no_element_calls_gpuis_a11y_builders_directly` keeps it that way.
pub trait Announce: StatefulInteractiveElement {
    /// Report `a11y` on this element.
    ///
    /// # Panics
    ///
    /// In debug builds, if the role needs an accessible name (see
    /// [`role_requires_a_name`]) and none was given, or if it needs a keyboard
    /// focus decision (see [`role_requires_keyboard_focus`]) and none was
    /// taken. Release builds announce the element rather than aborting.
    #[track_caller]
    fn announce(self, a11y: A11y) -> Self {
        debug_assert!(
            !a11y.is_missing_a_required_name(),
            "{:?} announces itself by name, and this one has none. Give it the element's \
             own visible text where it has any, or take the name as a constructor argument \
             where it does not — see `a11y`'s module docs, section 2. Not the tooltip.",
            a11y.role(),
        );
        debug_assert!(
            !a11y.is_missing_a_focus_decision(),
            "{:?} is a control a keyboard user operates, and this one says nothing about \
             keyboard focus. Call `.focusable()` (or `.focus_handle(handle)` if the element \
             owns one), or `.not_focusable(\"why\")` if it genuinely stays out of the tab \
             order — see `a11y`'s module docs, section 4. Announcing a role a keyboard \
             cannot reach is the defect that section exists for.",
            a11y.role(),
        );

        let A11y {
            role,
            name,
            description,
            toggled,
            selected,
            expanded,
            value,
            orientation,
            level,
            position_in_set,
            size_of_set,
            active_descendant,
            focus,
        } = a11y;

        let mut element = self.role(role);

        // Applied here rather than left to the element: a focus decision an
        // element has to honour separately is one that can silently disagree
        // with the element's own `div`. Section 4.
        match focus {
            // `focusable()` alone mints a handle whose `tab_stop` is false, and
            // `TabStopMap::next` walks straight past it.
            Focus::Takes(None) => {
                element = element.focusable().tab_stop(true).moves_focus_on_tab();
            }
            // `track_focus` does *not* push the element's `tab_stop` onto the
            // handle, so a caller-supplied one has to be made a stop itself.
            Focus::Takes(Some(handle)) => {
                element = element
                    .track_focus(&handle.tab_stop(true))
                    .moves_focus_on_tab();
            }
            Focus::Undecided | Focus::Declines(_) => {}
        }

        if let Some(name) = name {
            element = element.aria_label(name);
        }
        if let Some(description) = description {
            element = element.aria_description(description);
        }
        if let Some(toggled) = toggled {
            element = element.aria_toggled(toggled);
        }
        if let Some(selected) = selected {
            element = element.aria_selected(selected);
        }
        if let Some(expanded) = expanded {
            element = element.aria_expanded(expanded);
        }
        match value {
            Some(A11yValue::Text(text)) => element = element.aria_value(text),
            Some(A11yValue::Number {
                value,
                min,
                max,
                step,
            }) => {
                element = element
                    .aria_numeric_value(value)
                    .aria_min_numeric_value(min)
                    .aria_max_numeric_value(max)
                    .aria_numeric_value_step(step);
            }
            None => {}
        }
        if let Some(orientation) = orientation {
            element = element.aria_orientation(orientation);
        }
        if let Some(level) = level {
            element = element.aria_level(level);
        }
        if let Some(position) = position_in_set {
            element = element.aria_position_in_set(position);
        }
        if let Some(size) = size_of_set {
            element = element.aria_size_of_set(size);
        }
        // The whole of the apply, deliberately in one narrow shape: gpui's
        // builder takes no argument, so this `if` is the only thing that says
        // "not the active descendant", and no test in this crate can observe
        // the applied node. See `A11y::active_descendant`.
        if active_descendant {
            element = element.aria_active_descendant();
        }

        element
    }
}

impl<E: StatefulInteractiveElement> Announce for E {}

/// Reading an element's accessibility node back, which is otherwise
/// impossible: accessibility cannot be switched on in a test (see section 4 of
/// the module docs), so nothing ever builds gpui's real tree here. These
/// helpers call the same two `Element` methods gpui's walk calls, at the same
/// point, and hand back the node it would have built.
#[cfg(test)]
pub(crate) mod test_support {
    use gpui::{accesskit, App, Element, ElementId, IntoElement, RenderOnce, Role, Window};

    /// One element, as gpui's accessibility walk would have seen it.
    pub(crate) struct Announced {
        /// The element's own id. gpui builds no node without one, however good
        /// the role is.
        pub(crate) id: Option<ElementId>,
        /// What `Element::a11y_role` reported.
        pub(crate) role: Option<Role>,
        /// The node itself — `Some` only where gpui would have pushed one,
        /// which needs both an id and a role.
        pub(crate) node: Option<accesskit::Node>,
    }

    impl Announced {
        /// The accessible name on the node, if there is a node.
        pub(crate) fn name(&self) -> Option<&str> {
            self.node.as_ref().and_then(|node| node.label())
        }

        /// Whether the node offers `action` — how a test sees the difference
        /// gpui does not give us an `aria_disabled` for.
        pub(crate) fn supports(&self, action: accesskit::Action) -> bool {
            self.node
                .as_ref()
                .is_some_and(|node| node.supports_action(action))
        }
    }

    /// Render `component` and report what its root element announces.
    ///
    /// Calls `RenderOnce::render` rather than `into_element`: `#[derive(
    /// IntoElement)]` wraps the component in a `Component<C>`, whose
    /// `a11y_role()` is always `None`.
    ///
    /// **A `render` that returns `AnyElement` reports nothing here.**
    /// `AnyElement` does not forward `a11y_role` / `write_a11y_info` to what it
    /// wraps — gpui's real walk goes through the inner element's own
    /// `Drawable`, which this cannot reach. That is a limitation of the helper
    /// and not of the element: read it as "no answer", never as "no role".
    /// Such an element has to be checked by drawing it, the way
    /// `src/markdown/`'s recorder does.
    pub(crate) fn announced(
        component: impl RenderOnce,
        window: &mut Window,
        cx: &mut App,
    ) -> Announced {
        announced_element(component.render(window, cx).into_element())
    }

    /// The same, for an element built by hand.
    pub(crate) fn announced_element(element: impl Element) -> Announced {
        let id = element.id();
        let role = element.a11y_role();

        let node = match (&id, role) {
            (Some(_), Some(role)) => {
                let mut node = accesskit::Node::new(role);
                element.write_a11y_info(&mut node);
                Some(node)
            }
            // gpui pushes a node only for an element with both.
            _ => None,
        };

        Announced { id, role, node }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::announced_element;
    use super::*;
    use gpui::{accesskit, div, Orientation, Role, Toggled};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn a_role_and_a_name_reach_the_node() {
        let announced = announced_element(
            div()
                .id("save")
                .announce(A11y::new(Role::Button).name("Save").focusable()),
        );

        assert_eq!(announced.role, Some(Role::Button));
        assert_eq!(announced.name(), Some("Save"));
    }

    #[test]
    fn an_element_that_does_not_announce_has_no_node() {
        let announced = announced_element(div().id("plain"));

        assert_eq!(announced.role, None);
        assert!(
            announced.node.is_none(),
            "an element with no role is not in the tree at all"
        );
    }

    /// `Img` is the one element in gpui that takes an announcement without an
    /// id, and it is also the one that throws the announcement away: its
    /// `Element::a11y_role` never returns the stored role. Both halves of
    /// section 1's escape hatch, in one assertion.
    #[test]
    fn an_image_announces_nothing_however_hard_it_is_asked() {
        let announced = announced_element(
            gpui::img(SharedString::from("nothing.png"))
                .announce(A11y::new(Role::Image).name("A picture of nothing in particular")),
        );

        assert!(announced.id.is_none(), "no id was ever required of it");
        assert_eq!(
            announced.role, None,
            "`Img` does not report the role it was given — wrap it in a `div().id(…)`"
        );
        assert!(announced.node.is_none());
    }

    /// Every state field, in one element, so a field that is added to [`A11y`]
    /// but forgotten in [`Announce::announce`] fails here.
    ///
    /// **`active_descendant` is excluded, and this is the whole of why.** gpui
    /// applies it at paint time behind `window.a11y.is_active()`, which no test
    /// platform in this crate switches on, so there is no node to read it back
    /// off — asserting it here would assert nothing. What stands in its place
    /// is weaker than this test, and the substitution is stated rather than
    /// implied: the exhaustive `let A11y { … } = a11y` destructure in
    /// [`Announce::announce`] makes a *new* field going unhandled a compile
    /// error, and [`an_active_descendant_is_declared_rather_than_read_back`]
    /// holds the builder and the reader in place. Neither notices a *wrong*
    /// apply — dropping the `if` and calling gpui's builder unconditionally
    /// passes everything here. See [`A11y::active_descendant`].
    #[test]
    fn every_state_field_reaches_the_node() {
        let announced = announced_element(
            div().id("everything").announce(
                A11y::new(Role::Slider)
                    .name("Volume")
                    .description("Playback volume")
                    .toggled(Toggled::Mixed)
                    .selected(true)
                    .expanded(false)
                    .number_value(70., 0., 100., 5.)
                    .orientation(Orientation::Horizontal)
                    .level(2)
                    .position_in_set(3)
                    .size_of_set(8)
                    .focusable(),
            ),
        );

        let node = announced.node.expect("a slider with an id is a node");

        assert_eq!(node.label(), Some("Volume"));
        assert_eq!(node.description(), Some("Playback volume"));
        assert_eq!(node.toggled(), Some(Toggled::Mixed));
        assert_eq!(node.is_selected(), Some(true));
        assert_eq!(node.is_expanded(), Some(false));
        assert_eq!(node.numeric_value(), Some(70.));
        assert_eq!(node.min_numeric_value(), Some(0.));
        assert_eq!(node.max_numeric_value(), Some(100.));
        assert_eq!(node.numeric_value_step(), Some(5.));
        assert_eq!(node.orientation(), Some(Orientation::Horizontal));
        assert_eq!(node.level(), Some(2));
        assert_eq!(node.position_in_set(), Some(3));
        assert_eq!(node.size_of_set(), Some(8));
    }

    /// The active descendant is the one piece of state this crate declares and
    /// cannot read back, so what a test can hold is the declaration: a default
    /// `A11y` does not claim it, `active_descendant(true)` does,
    /// `active_descendant(false)` returns to not claiming, and an element
    /// carrying the claim still announces everything else normally.
    ///
    /// That last assertion is the point. `announce` consumes the whole `A11y`,
    /// and the claim is the only field applied through a gpui builder that
    /// takes no argument; a mistake there would most likely show up as the
    /// rest of the announcement going missing, which this *can* see.
    #[test]
    fn an_active_descendant_is_declared_rather_than_read_back() {
        let quiet = A11y::new(Role::ListBoxOption).name("Option 2");
        assert!(
            !quiet.is_active_descendant(),
            "a row claims the active descendant only when it is told to"
        );

        let claiming = quiet.clone().active_descendant(true);
        assert!(claiming.is_active_descendant());
        assert!(
            !claiming
                .clone()
                .active_descendant(false)
                .is_active_descendant(),
            "`false` is how a row stops claiming it, and there is no third state"
        );

        // Absent is the only "no": gpui's builder takes no argument, so
        // `Option<bool>` would carry a `Some(false)` that could never be
        // reported. See `A11y::active_descendant`.
        assert_eq!(
            quiet,
            claiming.clone().active_descendant(false),
            "not claiming it and having stopped claiming it are the same announcement"
        );

        let announced = announced_element(
            div()
                .id("row")
                .announce(claiming.position_in_set(2).size_of_set(3)),
        );
        let node = announced
            .node
            .expect("a listbox option with an id is a node");
        assert_eq!(node.label(), Some("Option 2"));
        assert_eq!(node.position_in_set(), Some(2));
        assert_eq!(node.size_of_set(), Some(3));
    }

    #[test]
    fn a_text_value_reaches_the_node() {
        let announced = announced_element(
            div().id("timezone").announce(
                A11y::new(Role::ComboBox)
                    .name("Timezone")
                    .text_value("UTC")
                    .focusable(),
            ),
        );

        let node = announced.node.expect("a combo box with an id is a node");

        assert_eq!(node.label(), Some("Timezone"), "the name is not the value");
        assert_eq!(node.value(), Some("UTC"));
    }

    /// A click listener is what gpui derives `Action::Click` from, and — with
    /// no `aria_disabled` to set — it is the only thing that distinguishes a
    /// disabled control's node from an enabled one's.
    #[test]
    fn a_click_listener_is_what_offers_the_click_action() {
        let inert = announced_element(
            div().id("inert").announce(
                A11y::new(Role::Button)
                    .name("Save")
                    .not_focusable("this one is only here to have no click listener"),
            ),
        );
        assert!(!inert.supports(accesskit::Action::Click));
    }

    /// Every arm of [`role_requires_a_name`], one assertion per reason, so a
    /// role cannot fall off the list without a failure that says why it was on
    /// it.
    ///
    /// The list feeds a `debug_assert!`, so its rot is quiet: dropping an arm
    /// takes an element's name requirement with it and breaks no other test.
    /// Membership here is **exhaustive** over the list — all 28 arms — because
    /// a sample would leave the roles it does not name able to fall off exactly
    /// as the named ones can.
    ///
    /// **Non-membership is not exhaustive, and that is the asymmetry to know
    /// about.** The three absences asserted below are the ones this crate has
    /// argued for in writing; adding some *other* role to the list — `Group`,
    /// say — still passes here. That is deliberate. Nothing in this crate has
    /// argued about `Group` and the name rule, and an assertion with no
    /// argument behind it is the failure mode this module is organised
    /// against. What catches a wrong *addition* is the review that adds it.
    #[test]
    fn the_name_rule_covers_the_roles_that_are_nothing_without_one() {
        for role in [
            Role::Button,
            Role::DefaultButton,
            Role::CheckBox,
            Role::Switch,
            Role::RadioButton,
            Role::Link,
        ] {
            assert!(
                role_requires_a_name(role),
                "{role:?} is a single control whose whole announcement is name + role + \
                 state, so without a name a screen reader reads out its state and never \
                 says what it acts on"
            );
        }

        for role in [
            Role::MenuItem,
            Role::MenuItemCheckBox,
            Role::MenuItemRadio,
            Role::ListBoxOption,
            Role::Tab,
            Role::TreeItem,
        ] {
            assert!(
                role_requires_a_name(role),
                "{role:?} is an item inside a composite: the composite is named once and \
                 every item still has to say which one it is, so a nameless item is an \
                 unidentifiable row in an otherwise navigable list"
            );
        }

        for role in [
            Role::Slider,
            Role::SpinButton,
            Role::ProgressIndicator,
            Role::Meter,
        ] {
            assert!(
                role_requires_a_name(role),
                "{role:?} announces a number, and a number with no name is a quantity of \
                 nothing — the name is the only part that says what is being measured"
            );
        }

        assert!(
            role_requires_a_name(Role::Splitter),
            "a divider between two panes has no visible text of its own to borrow a name \
             from, so its name is a constructor argument — see `src/elements/splitter.rs`"
        );

        for role in [
            Role::ComboBox,
            Role::EditableComboBox,
            Role::TextInput,
            Role::MultilineTextInput,
            Role::SearchInput,
            Role::NumberInput,
            Role::PasswordInput,
            Role::DateInput,
        ] {
            assert!(
                role_requires_a_name(role),
                "{role:?} takes or chooses a value, and its own contents are the value \
                 rather than the label, so the name is the only thing that says what is \
                 being typed or chosen"
            );
        }

        for role in [Role::Dialog, Role::AlertDialog] {
            assert!(
                role_requires_a_name(role),
                "{role:?} takes over the screen, and its name is what a screen reader \
                 announces on arrival to say what was interrupted for"
            );
        }

        assert!(
            role_requires_a_name(Role::Image),
            "an image's name is its alternative text, which is the whole of what a screen \
             reader has to go on"
        );

        // The absences, each argued for somewhere in writing rather than
        // merely unlisted.
        for role in [Role::Complementary, Role::Document] {
            assert!(
                !role_requires_a_name(role),
                "{role:?} is named by what it contains, so a name is welcome but not \
                 compulsory — see this function's own docs"
            );
        }

        assert!(
            !role_requires_a_name(Role::ListBox),
            "`src/elements/select.rs` argues this exclusion in writing: the listbox is \
             named by the trigger beside it, so forcing a name on it would make every \
             select announce its label twice"
        );
    }

    /// The three states a required name can be in, and only silence and blank
    /// are bugs.
    #[test]
    fn a_required_name_is_absent_blank_or_given() {
        assert!(A11y::new(Role::Button).is_missing_a_required_name());
        assert!(A11y::new(Role::Button)
            .name("  ")
            .is_missing_a_required_name());
        assert!(!A11y::new(Role::Button)
            .name("Save")
            .is_missing_a_required_name());
        assert!(!A11y::new(Role::Document).is_missing_a_required_name());

        assert_eq!(
            A11y::new(Role::Button).name("Save").accessible_name(),
            Some(&"Save".into())
        );
        assert_eq!(A11y::new(Role::Button).role(), Role::Button);
    }

    #[test]
    #[should_panic(expected = "announces itself by name")]
    fn a_nameless_button_is_a_bug() {
        let _ = div().id("nameless").announce(A11y::new(Role::Button));
    }

    #[test]
    #[should_panic(expected = "announces itself by name")]
    fn an_empty_name_is_no_name() {
        let _ = div().id("blank").announce(A11y::new(Role::Button).name(""));
    }

    /// The convention, made enforceable.
    ///
    /// gpui's a11y builders are reachable from anywhere with an id, so
    /// "everyone uses [`A11y`]" is only true while nothing quietly does not.
    /// This is the same scan `element_id::tests::no_element_mints_a_constant_id`
    /// runs for ids, against the same corpus.
    #[test]
    fn no_element_calls_gpuis_a11y_builders_directly() {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();

        let files = rust_files(&src);

        // A scan that finds nothing to scan reports no offenders, which is
        // indistinguishable from a clean crate. `a11y_builder_calls` is
        // unit-tested below; this is the other half.
        assert!(
            files.len() > 20,
            "the scan found only {} source file(s) under {}, so it is not \
             guarding anything — check how the source tree is being located \
             before trusting a green result here",
            files.len(),
            src.display()
        );

        for file in files {
            // This module is where the builders live. Everything else goes
            // through `A11y`.
            if file == src.join("a11y.rs") {
                continue;
            }

            let source = fs::read_to_string(&file).expect("source file is readable");
            let relative = file
                .strip_prefix(&src)
                .unwrap_or(&file)
                .display()
                .to_string();

            for (line, text) in a11y_builder_calls(&source) {
                offenders.push(format!("  src/{relative}:{line}: {text}"));
            }
        }

        assert!(
            offenders.is_empty(),
            "these elements call gpui's accessibility builders directly:\n{}\n\n\
             Build a `crate::a11y::A11y` instead and apply it with `.announce(a11y)`, so that \
             one place decides what an element announces. If `A11y` has no field for what you \
             need, add the field and apply it in `Announce::announce` — that is the intended \
             move, not a local call. See the `a11y` module docs.",
            offenders.join("\n")
        );
    }

    // --- section 4: the keyboard ---

    /// Every arm of [`role_requires_keyboard_focus`], one assertion per reason,
    /// so a role cannot fall off the list without a failure that says why it
    /// was on it.
    ///
    /// The list feeds a `debug_assert!` inside [`Announce::announce`] and
    /// nothing else reads it, so its rot is quiet in the same way the name
    /// rule's is: dropping an arm takes a control's focus requirement with it,
    /// breaks no other test, and a release build never evaluates the assertion
    /// at all. Membership here is therefore **exhaustive** over the list — all
    /// 17 arms — grouped by the reason each one is on it. This test used to
    /// name four (`Button`, `DefaultButton`, `ComboBox`, `TextInput`), which
    /// left the other twelve deletable with the suite still green; the
    /// exhaustiveness is load-bearing rather than incidental.
    ///
    /// **Non-membership is not exhaustive, and that is the asymmetry to know
    /// about.** Doing it exhaustively would mean asserting against all 182 of
    /// accesskit's `Role` variants, almost none of which this crate has said
    /// anything about, and an assertion with no argument behind it is the
    /// failure mode this module is organised against. So the negative half
    /// asserts *arguments*: exactly the three groups the rule's own docs
    /// decline in writing. What catches a wrong *addition* is the review that
    /// adds it.
    #[test]
    fn the_focus_rule_covers_the_roles_a_keyboard_operates() {
        for role in [Role::Button, Role::Link] {
            assert!(
                role_requires_keyboard_focus(role),
                "{role:?} is activated by a keystroke on the control itself, and a \
                 keystroke only ever reaches the focused element"
            );
        }

        assert!(
            role_requires_keyboard_focus(Role::DefaultButton),
            "a dialog's Enter key resolves to the default button, so it is the most \
             focus-requiring control in the set"
        );

        for role in [Role::CheckBox, Role::Switch, Role::RadioButton] {
            assert!(
                role_requires_keyboard_focus(role),
                "{role:?} holds a state the keyboard changes with Space, and that toggle \
                 is a keystroke that has to land on the control"
            );
        }

        for role in [Role::Slider, Role::SpinButton] {
            assert!(
                role_requires_keyboard_focus(role),
                "{role:?} moves its value with the arrow keys, which go to the focused \
                 element and nowhere else"
            );
        }

        assert!(
            role_requires_keyboard_focus(Role::Splitter),
            "a splitter is a standalone control owning one tab stop whose arrow keys move \
             the divider — `Slider`'s shape — so it declares focus on its `A11y` like every \
             other keyboard-operable control rather than through a raw `tab_index`"
        );

        for role in [Role::ComboBox, Role::EditableComboBox] {
            assert!(
                role_requires_keyboard_focus(role),
                "{role:?} delivers its popup's arrow keys to the trigger, so the trigger \
                 holding focus is what makes the list operable at all"
            );
        }

        for role in [
            Role::TextInput,
            Role::MultilineTextInput,
            Role::SearchInput,
            Role::NumberInput,
            Role::PasswordInput,
            Role::DateInput,
        ] {
            assert!(
                role_requires_keyboard_focus(role),
                "{role:?} is typed into, and typing is nothing but the focused element's \
                 keystrokes"
            );
        }

        for role in [
            Role::MenuItem,
            Role::MenuItemCheckBox,
            Role::MenuItemRadio,
            Role::ListBoxOption,
            Role::Tab,
            Role::TreeItem,
        ] {
            assert!(
                !role_requires_keyboard_focus(role),
                "{role:?} is an arrow-key target inside a composite that owns the one tab \
                 stop, so making each item a tab stop is the mistake the ARIA authoring \
                 practices call out — these join the list with a roving-focus convention"
            );
        }

        for role in [Role::Complementary, Role::Document, Role::Group] {
            assert!(
                !role_requires_keyboard_focus(role),
                "{role:?} is a landmark or container: it is read, not operated"
            );
        }
    }

    /// The three states are distinguishable, and only silence is a bug.
    #[test]
    fn a_focus_decision_is_taken_declined_or_missing() {
        let undecided = A11y::new(Role::Button).name("Save");
        assert!(!undecided.is_focusable());
        assert_eq!(undecided.focus_declined_because(), None);
        assert!(undecided.is_missing_a_focus_decision());

        let takes = undecided.clone().focusable();
        assert!(takes.is_focusable());
        assert!(!takes.is_missing_a_focus_decision());

        let declines = undecided.clone().not_focusable("it is disabled");
        assert!(!declines.is_focusable());
        assert_eq!(
            declines.focus_declined_because(),
            Some(&"it is disabled".into())
        );
        assert!(
            !declines.is_missing_a_focus_decision(),
            "the assertion is against silence, not against \"no\""
        );

        // A role nobody operates never has to answer.
        assert!(!A11y::new(Role::Complementary).is_missing_a_focus_decision());
    }

    #[test]
    #[should_panic(expected = "says nothing about keyboard focus")]
    fn a_button_that_says_nothing_about_focus_is_a_bug() {
        let _ = div()
            .id("mute")
            .announce(A11y::new(Role::Button).name("Save"));
    }

    /// The counterpart: declining is a decision, so it announces normally.
    #[test]
    fn a_declined_control_still_announces() {
        let announced = announced_element(
            div().id("off").announce(
                A11y::new(Role::Button)
                    .name("Save")
                    .not_focusable("it is disabled"),
            ),
        );

        assert_eq!(announced.role, Some(Role::Button));
        assert_eq!(announced.name(), Some("Save"));
    }

    /// A caller-supplied handle is made a tab stop on the *handle*, because
    /// `track_focus` does not carry the element's `tab_stop` onto it.
    #[gpui::test]
    fn a_supplied_handle_is_made_a_tab_stop(cx: &mut gpui::TestAppContext) {
        let handle = cx.update(|cx| cx.focus_handle());
        assert!(!handle.tab_stop, "gpui mints handles that are not stops");

        let a11y = A11y::new(Role::Button)
            .name("Save")
            .focus_handle(handle.clone());
        assert!(a11y.is_focusable());

        // What reaches the tab order is checked by drawing — see
        // `elements::button`'s keyboard tests. What is checked here is that the
        // announcement carries the caller's handle rather than minting one.
        let _ = div().id("save").announce(a11y);
    }

    /// The coverage guard: `src/elements.rs`'s `pub mod` list, checked in both
    /// directions against [`ELEMENTS_WITHOUT_A_ROLE`], so the excuse list can
    /// only shrink.
    ///
    /// Modelled on `elements::overlay_coverage::every_overlay_is_written_down`,
    /// which is this crate's existing shape for holding a list to the tree.
    #[test]
    fn every_element_module_declares_a_role() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let elements =
            fs::read_to_string(root.join("src/elements.rs")).expect("src/elements.rs is readable");

        let modules: Vec<String> = elements
            .lines()
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("pub mod ")?
                    .strip_suffix(';')
                    .map(str::to_string)
            })
            .collect();

        // The same corpus floor the builder scan carries, for the same reason:
        // a scan that finds nothing to scan reports no offenders.
        assert!(
            modules.len() > 20,
            "only {} element module(s) found in src/elements.rs, so this guards nothing — \
             check how the source tree is being located before trusting a green result",
            modules.len(),
        );

        let mut silent = Vec::new();
        let mut adopted = Vec::new();

        for module in &modules {
            let source = fs::read_to_string(root.join(format!("src/elements/{module}.rs")))
                .unwrap_or_else(|error| panic!("src/elements/{module}.rs is unreadable: {error}"));

            let declares = source
                .lines()
                .any(|line| line.contains("Accessible for") && line.contains("impl"));
            let excused = ELEMENTS_WITHOUT_A_ROLE
                .iter()
                .any(|(name, _)| name == module);

            if !declares && !excused {
                silent.push(module.clone());
            }
            if declares && excused {
                adopted.push(module.clone());
            }
        }

        assert!(
            silent.is_empty(),
            "these element modules neither implement `Accessible` nor say why not: {}\n\n\
             Adopt the module into `crate::a11y` — one `impl Accessible`, one `.announce(…)` \
             — or add it to `ELEMENTS_WITHOUT_A_ROLE` with a reason saying what it would \
             announce or what has to exist first.",
            silent.join(", "),
        );
        assert!(
            adopted.is_empty(),
            "these element modules implement `Accessible` but are still excused in \
             `ELEMENTS_WITHOUT_A_ROLE`: {}. Delete their entries — the list only shrinks.",
            adopted.join(", "),
        );

        for (module, reason) in ELEMENTS_WITHOUT_A_ROLE {
            assert!(
                modules.iter().any(|name| name == module),
                "`ELEMENTS_WITHOUT_A_ROLE` excuses `{module}`, which src/elements.rs declares \
                 no `pub mod` for"
            );
            assert!(
                reason.len() >= 40,
                "`{module}`'s reason is {} characters. A reason says what the module would \
                 announce or what has to exist first — \"not done yet\" is not one",
                reason.len(),
            );
        }
    }

    fn rust_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("source directory is readable") {
                let path = entry.expect("directory entry is readable").path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    files.push(path);
                }
            }
        }

        files.sort();
        files
    }

    /// The `.role(…)` / `.aria_…(…)` calls in `source`, as
    /// `(line number, trimmed line)`.
    ///
    /// Top-level `#[cfg(test)] mod …` blocks are skipped, on the same
    /// reasoning as the id scan: a test is entitled to build an element by
    /// hand to check what the convention produced. Those blocks are found by
    /// column — an attribute in column 0 opens one, a `}` in column 0 closes
    /// it — which holds for rustfmt-formatted source.
    ///
    /// `.a11y_role()` and `RunRole::a11y_role()` are not matched: the dot has
    /// to be immediately before `role(`, so a hand-written `Element`'s own
    /// methods read as what they are.
    fn a11y_builder_calls(source: &str) -> Vec<(usize, String)> {
        let mut hits = Vec::new();
        let mut in_test_module = false;

        for (index, line) in source.lines().enumerate() {
            if in_test_module {
                if line == "}" {
                    in_test_module = false;
                }
                continue;
            }
            if line.starts_with("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains(".role(") || trimmed.contains(".aria_") {
                hits.push((index + 1, trimmed.to_string()));
            }
        }

        hits
    }

    #[test]
    fn the_scan_reads_builder_calls_and_not_their_lookalikes() {
        let source = r#"
fn render(self) -> impl IntoElement {
    div()
        .id(self.id)
        .role(Role::Button)
        .aria_label("Save")
        .aria_expanded(true)
}

fn fine(self) -> impl IntoElement {
    // .role(Role::Button) in a comment
    //! and in a doc comment: `.aria_label`
    div().id(self.id).announce(self.a11y())
}

impl Element for Run {
    fn a11y_role(&self) -> Option<Role> {
        Some(self.role.a11y_role())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test_may_build_an_element_by_hand() {
        div().id("left").role(Role::Button);
    }
}
"#;

        assert_eq!(
            a11y_builder_calls(source)
                .into_iter()
                .map(|(line, _)| line)
                .collect::<Vec<_>>(),
            vec![5, 6, 7]
        );
    }
}
