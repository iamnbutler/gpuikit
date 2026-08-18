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
//!         A11y::new(Role::Button).name(self.label.clone())
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
//! `position_in_set`. **State goes on the element that changes it**, which is
//! not always the element it is about: `Sidebar` reports `aria-expanded` on
//! `SidebarTrigger`, because the trigger is the control a screen reader user
//! operates, and the panel is the thing that happens as a result.
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
//! # 4. How it is tested
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
//! What the scan cannot see is a hand-written [`gpui::Element`] writing an
//! `accesskit::Node` itself — `src/markdown/selectable_text.rs` does, because
//! a text run is not a `div` and never passes through a builder. That is the
//! one place in the crate outside this convention, and it has its own tests.
//!
//! # 5. The rollout
//!
//! `Button` is the worked example, and `Sidebar` — which shipped a role ahead
//! of this decision — has been migrated onto it, as its own issue said it
//! would have to be. Nothing else is swept: the state fields are proven
//! against a bare `div` in this module's tests rather than by touching thirty
//! elements. The order the follow-on work wants is `IconButton` (which forces
//! the "name as a constructor argument" half of section 2), then
//! `Checkbox` / `Switch` / `Toggle`, `Slider` / `Progress`, `Tabs` / `List`,
//! `Accordion` / `Collapsible`, the overlays, and `Table` last, since it needs
//! derived cell ids first.

use gpui::{Orientation, Role, SharedString, StatefulInteractiveElement, Toggled};

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
}

/// Whether a role's whole announcement is *name + role + state*, so that
/// leaving the name out leaves the element unidentifiable.
///
/// This is a list, and lists rot. It is public so that an element — or its
/// test — can consult the same one, and it is expected to grow as elements are
/// adopted. A role that is missing from it is not *forbidden* a name; it is
/// only not forced to have one, which is right for a landmark or a container
/// that is named by its contents (`Role::Complementary`, `Role::Document`).
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
    /// [`role_requires_a_name`]) and none was given. Release builds announce
    /// the nameless element rather than aborting.
    #[track_caller]
    fn announce(self, a11y: A11y) -> Self {
        debug_assert!(
            !a11y.is_missing_a_required_name(),
            "{:?} announces itself by name, and this one has none. Give it the element's \
             own visible text where it has any, or take the name as a constructor argument \
             where it does not — see `a11y`'s module docs, section 2. Not the tooltip.",
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
        } = a11y;

        let mut element = self.role(role);

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
    use gpui::{accesskit, div, prelude::*, Orientation, Role, Toggled};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn a_role_and_a_name_reach_the_node() {
        let announced = announced_element(
            div()
                .id("save")
                .announce(A11y::new(Role::Button).name("Save")),
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
                    .position_in_set(3),
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
    }

    #[test]
    fn a_text_value_reaches_the_node() {
        let announced = announced_element(
            div()
                .id("timezone")
                .announce(A11y::new(Role::ComboBox).name("Timezone").text_value("UTC")),
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
            div()
                .id("inert")
                .announce(A11y::new(Role::Button).name("Save")),
        );
        assert!(!inert.supports(accesskit::Action::Click));
    }

    #[test]
    fn the_naming_rule_covers_the_roles_that_name_themselves() {
        assert!(role_requires_a_name(Role::Button));
        assert!(role_requires_a_name(Role::CheckBox));
        // A landmark is named by what it contains, so a name is welcome but
        // not compulsory.
        assert!(!role_requires_a_name(Role::Complementary));
        assert!(!role_requires_a_name(Role::Document));

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
