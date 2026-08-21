//! Grouping and label association — and deliberately nothing else.
//!
//! This module is #164's answer, and its title is the whole of the scope:
//! *grouping and label association, not form state*. There is no submission,
//! no validation, no dirty tracking and no form-level value store here.
//! Every one of those is a decision about the consuming application's
//! architecture, and a toolkit that takes it is one people fight rather than
//! use — the line Headless UI draws, and the one #164 argued for.
//!
//! What it does provide is two things a control genuinely cannot work out for
//! itself:
//!
//! - **A group.** [`Fieldset`] draws a legend, a description and a
//!   *group-level* error — the summary that previously had nowhere to go,
//!   because [`Field`](crate::elements::field::Field)'s error is about one
//!   control — and announces itself as a named `Role::Group`.
//! - **An ambient [`FormContext`].** A `Fieldset`'s `disabled` reaches every
//!   control inside it, and a `Field`'s label reaches the control beside it,
//!   without either being threaded through a prop by hand.
//!
//! # Why the context is ambient
//!
//! A `Field`'s child is an `AnyElement`: by the time the field has it, the
//! control inside is opaque and no builder can reach it. Threading a prop
//! would mean every container taking a `disabled` and every caller passing it
//! down — which is the thing the cascade exists to remove, reintroduced one
//! call site at a time.
//!
//! The mechanism is a thread-local stack, pushed around the child's layout and
//! paint by [`WithFormContext`]. It works because gpui renders a subtree
//! *inside* its ancestor's layout pass: `ViewElement::request_layout` calls
//! `RenderOnce::render` and then lays out the result, and `Div` does the same
//! for each of its children, so a whole descendant subtree is built while the
//! ancestor's `request_layout` is on the Rust stack. The scope is opened in
//! `prepaint` and `paint` too, since those walk the subtree again.
//!
//! A `gpui::Global` was considered and rejected: reading one needs `&mut App`,
//! which puts a `cx` argument on [`disabled_here`] and takes the ambient value
//! straight back to being threaded by hand.
//!
//! # Reading it from a control
//!
//! One line in `render`, before the element's fields are moved:
//!
//! ```ignore
//! let disabled = form::disabled_here(self.disabled);
//! ```
//!
//! `render` runs during `request_layout`, which is *inside* the scope. That is
//! the only place a control should read it.
//!
//! ## The one rule about out-of-line draws
//!
//! **Read the ambient value in `render` and pass what you read into anything
//! you draw out of line; never call [`disabled_here`] from inside a
//! draw-deferring closure.** `Window::defer_draw` clones and restores gpui's
//! *element-id* stack, not this one, so a closure that runs after the frame's
//! layout has unwound reads an empty stack and gets `false`.
//!
//! This is narrower than "out-of-line draws are broken". A popover inside a
//! disabled `Fieldset` reads `disabled_here` in its own `render` — inside the
//! scope, correct — and captures the answer; only a read performed *within*
//! the closure sees nothing. Hand-threading `disabled(true)` into every popup
//! is not the fix, and would be the threading this design exists to remove.
//!
//! The case that genuinely cannot be handled this way is a closure that is
//! built once and drawn against a *different* ambient value later — a panel
//! cached across frames, or one whose content is produced by a callback the
//! containing element stores rather than calls during layout. Such a callback
//! has to be given the value as an argument, because there is no frame in
//! which it and the group are on the stack together.

use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;

use crate::a11y::{A11y, Announce};
use crate::layout::v_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, Bounds, ElementId, FocusHandle,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, ParentElement, Pixels, RenderOnce,
    InteractiveElement, Role, SharedString, Styled, Window,
};

/// What a group tells the controls inside it.
///
/// Three fields, each answering a question a control cannot answer alone:
/// whether an enclosing group has disabled it, what an enclosing
/// [`Field`](crate::elements::field::Field) calls it, and which focus handle
/// that field's label clicks.
#[derive(Clone, Default, PartialEq)]
pub struct FormContext {
    /// Whether an enclosing group has disabled everything inside it.
    pub disabled: bool,
    /// The accessible name an enclosing field publishes for the control beside
    /// its label.
    pub name: Option<SharedString>,
    /// The handle a label click focuses, for the control that agrees to track
    /// it.
    pub focus_handle: Option<FocusHandle>,
}

impl std::fmt::Debug for FormContext {
    /// `FocusHandle` is not `Debug`, and the useful fact about it here is
    /// whether there is one at all.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormContext")
            .field("disabled", &self.disabled)
            .field("name", &self.name)
            .field("focus_handle", &self.focus_handle.is_some())
            .finish()
    }
}

impl FormContext {
    /// An empty context — nothing disabled, nothing named.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable everything in this scope.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Publish an accessible name for the control in this scope.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Publish the focus handle a label click should land on.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// This context as it reads when nested inside `outer`.
    ///
    /// `disabled` is OR'd: an inner scope can disable, and cannot re-enable.
    /// A `Fieldset` that says nothing about `disabled` inside a disabled one
    /// is still disabled, and a `Field` cannot rescue a control from the group
    /// around it — which is the only reading of "the group is disabled" that
    /// is worth anything.
    ///
    /// `name` and `focus_handle` are *inherited when unset*: the nearest
    /// enclosing answer wins, and a scope that has none passes the outer one
    /// through.
    pub fn over(mut self, outer: &FormContext) -> Self {
        self.disabled |= outer.disabled;
        if self.name.is_none() {
            self.name = outer.name.clone();
        }
        if self.focus_handle.is_none() {
            self.focus_handle = outer.focus_handle.clone();
        }
        self
    }
}

thread_local! {
    /// The open scopes, innermost last.
    ///
    /// Thread-local rather than global: gpui builds a window's element tree on
    /// one thread, and a value that crossed threads would be a value two
    /// windows could disagree about.
    static SCOPES: RefCell<Vec<FormContext>> = const { RefCell::new(Vec::new()) };
}

/// An open scope, which closes when this is dropped.
///
/// A guard rather than a matched pair of calls: a child that panics mid-layout
/// would otherwise leave the stack one deep for every element drawn after it,
/// and the symptom — an unrelated control announcing itself disabled — would
/// point nowhere near the cause.
#[must_use = "the scope closes as soon as this guard is dropped"]
pub struct FormScope {
    _private: (),
}

impl Drop for FormScope {
    fn drop(&mut self) {
        SCOPES.with(|scopes| {
            scopes.borrow_mut().pop();
        });
    }
}

/// Open a scope carrying `context`, nested inside whatever is already open.
///
/// Public because [`WithFormContext`] is not the only thing that could
/// legitimately open one — a container that is a hand-written `Element`
/// already can — but a control should never need to call it.
pub fn push(context: FormContext) -> FormScope {
    SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        let nested = match scopes.last() {
            Some(outer) => context.over(outer),
            None => context,
        };
        scopes.push(nested);
    });
    FormScope { _private: () }
}

/// Run `f` inside a scope carrying `context`.
pub fn scope<R>(context: FormContext, f: impl FnOnce() -> R) -> R {
    let _guard = push(context);
    f()
}

/// The innermost open [`FormContext`], if any.
pub fn current() -> Option<FormContext> {
    SCOPES.with(|scopes| scopes.borrow().last().cloned())
}

/// Whether a control with its own `own` disabled flag is disabled here.
///
/// The whole of a control's adoption cost. `own` wins where it is `true`, and
/// the ambient value supplies the answer where it is not.
pub fn disabled_here(own: bool) -> bool {
    own || SCOPES.with(|scopes| scopes.borrow().last().is_some_and(|scope| scope.disabled))
}

/// The accessible name an enclosing group publishes, if any.
///
/// A control announces this only when it has no name of its own — its own
/// visible text always wins, per `a11y`'s section 2.
pub fn name_here() -> Option<SharedString> {
    SCOPES.with(|scopes| scopes.borrow().last().and_then(|scope| scope.name.clone()))
}

/// The focus handle an enclosing label clicks, if any.
///
/// A control that tracks this is one whose label can be clicked to focus it.
/// A control that does not simply leaves the click inert.
pub fn focus_handle_here() -> Option<FocusHandle> {
    SCOPES.with(|scopes| {
        scopes
            .borrow()
            .last()
            .and_then(|scope| scope.focus_handle.clone())
    })
}

thread_local! {
    /// One [`FocusHandle`] per field id, so that the handle a label publishes
    /// in one frame is the handle a control tracked in the last one.
    ///
    /// `Window::use_keyed_state` — the obvious home for this — cannot be used:
    /// it goes through `with_element_state`, which `debug_assert!`s that it is
    /// called during prepaint or paint, and `RenderOnce::render` runs during
    /// `request_layout`.
    ///
    /// **This never evicts.** Its size is the number of *distinct field ids
    /// ever rendered on this thread*, not the number of fields on screen. For
    /// a form whose ids are written down in the source that is a fixed, small
    /// number. For a caller that derives a field id from a row, a record or a
    /// task — the ordinary way an id gets made in a list-driven application —
    /// it grows without limit, one `FocusHandle` per id, for the life of the
    /// process.
    ///
    /// Evicting an entry whose only remaining reference is this map was
    /// considered and not done: `FocusHandle` exposes no reference count, so
    /// "nobody else holds this" is not a question that can be asked from here,
    /// and dropping a handle a control still tracks would silently unfocus it.
    /// The fix that is actually available is an explicit one — a
    /// `clear_field_focus_handles()` a long-lived list view calls when its
    /// backing collection changes — and it is not written until something
    /// needs it.
    static FIELD_FOCUS_HANDLES: RefCell<HashMap<ElementId, FocusHandle>> =
        RefCell::new(HashMap::new());
}

/// The focus handle belonging to the field with this id, minting one the first
/// time it is asked for.
///
/// Stable across frames, which is the whole point: a label click focuses the
/// same handle the control tracked when it was drawn.
pub fn field_focus_handle(id: &ElementId, cx: &mut App) -> FocusHandle {
    FIELD_FOCUS_HANDLES.with(|handles| {
        let mut handles = handles.borrow_mut();
        handles
            .entry(id.clone())
            .or_insert_with(|| cx.focus_handle())
            .clone()
    })
}

/// Wraps one child so that everything drawn inside it sees `context`.
///
/// The one hand-written [`gpui::Element`] under `src/elements/`, and it is one
/// because the scope has to be open across `request_layout`, `prepaint` and
/// `paint` — three moments a `RenderOnce` cannot get between. It reports no id
/// and no role, so it adds nothing to the accessibility tree and nothing to
/// the element-id path: wrapping a subtree in it cannot change what any
/// element inside it is called.
pub struct WithFormContext {
    context: FormContext,
    child: AnyElement,
}

impl WithFormContext {
    /// Wrap `child` so that it, and everything it draws, reads `context`.
    pub fn new(context: FormContext, child: impl IntoElement) -> Self {
        Self {
            context,
            child: child.into_any_element(),
        }
    }
}

impl IntoElement for WithFormContext {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

// `gpui::Element` rather than an imported `Element`: the trait's own `id(&self)`
// would otherwise shadow `InteractiveElement::id(self, …)` on every `div()` in
// this module.
impl gpui::Element for WithFormContext {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let _scope = push(self.context.clone());
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let _scope = push(self.context.clone());
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let _scope = push(self.context.clone());
        self.child.paint(window, cx);
    }
}

/// A group of related controls, with a legend and a `disabled` that cascades.
///
/// # Example
///
/// ```ignore
/// fieldset("billing")
///     .legend("Billing address")
///     .error("This address could not be verified")
///     .disabled(!self.editable)
///     .child(field("street").label("Street").child(street_input))
///     .child(field("city").label("City").child(city_input))
/// ```
///
/// The `disabled(!self.editable)` is the point: neither field, and neither
/// control inside them, says anything about `disabled` at all.
#[derive(IntoElement)]
pub struct Fieldset {
    id: ElementId,
    legend: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    disabled: bool,
    size: ControlSize,
    children: Vec<AnyElement>,
}

impl Fieldset {
    /// A fieldset with the given id.
    ///
    /// The id is required, and required to be unique among everything drawn in
    /// a frame — see [`crate::element_id`]. It carries the group's
    /// accessibility node.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            legend: None,
            description: None,
            error: None,
            disabled: false,
            size: ControlSize::default(),
            children: Vec::new(),
        }
    }

    /// The group's visible heading, which is also its accessible name.
    pub fn legend(mut self, legend: impl Into<SharedString>) -> Self {
        self.legend = Some(legend.into());
        self
    }

    /// Help text for the group as a whole.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// An error about the group rather than about one control in it.
    ///
    /// "These dates overlap" belongs to neither date field. Before this there
    /// was nowhere to put it but one of the two, which said something untrue
    /// about that control.
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// Creates a new [`Fieldset`] with the given id.
pub fn fieldset(id: impl Into<ElementId>) -> Fieldset {
    Fieldset::new(id)
}

impl Disableable for Fieldset {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Disables **everything inside the group**, through the ambient
    /// [`FormContext`] — not only the legend and the error this element draws
    /// itself. A control inside that reads [`disabled_here`] needs no
    /// `disabled` of its own.
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for Fieldset {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl ParentElement for Fieldset {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Accessible for Fieldset {
    /// A named `Role::Group`. `Group` is read rather than operated, so it
    /// needs neither a required name nor a focus decision — but a legend is a
    /// name, and where there is one it is the group's.
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::Group);
        match &self.legend {
            Some(legend) => a11y.name(legend.clone()),
            None => a11y,
        }
    }
}

impl RenderOnce for Fieldset {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let a11y = self.a11y();
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        // Its own `disabled`, over any group already around it: a fieldset
        // inside a disabled fieldset is disabled, and dims its own legend to
        // say so.
        let disabled = disabled_here(self.disabled);

        let legend = self.legend.map(|legend| {
            div()
                .text_size(metrics.text_size)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(if disabled {
                    theme.fg_disabled()
                } else {
                    theme.fg()
                })
                .child(legend)
        });

        let description = self.description.map(|description| {
            div()
                .text_xs()
                .text_color(if disabled {
                    theme.fg_disabled()
                } else {
                    theme.fg_muted()
                })
                .child(description)
        });

        let error = self
            .error
            .map(|error| div().text_xs().text_color(theme.danger()).child(error));

        // The legend, the description and the error sit *outside* the context
        // wrapper. They are the group talking about itself, not controls in
        // it, and an error that dimmed itself when the group was disabled
        // would be least readable exactly when it mattered most.
        v_stack()
            .id(self.id)
            .announce(a11y)
            .gap(rems(0.5))
            .when_some(legend, |this, legend| this.child(legend))
            .when_some(description, |this, description| this.child(description))
            .child(WithFormContext::new(
                FormContext::new().disabled(disabled),
                v_stack().gap(rems(0.75)).children(self.children),
            ))
            .when_some(error, |this, error| this.child(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::test_support::announced;
    use crate::elements::checkbox::{checkbox, Checkbox};
    use crate::elements::field::field;
    use crate::traits::labelable::Labelable;
    use gpui::{
        px, size, AnyElement, AppContext, Bounds, Context, Modifiers, Render, TestAppContext,
        VisualTestContext,
    };

    /// Draws whatever the test's closure builds, every frame.
    ///
    /// A real *view* rather than `VisualTestContext::draw`: registering a
    /// mouse listener reads `Window::current_view`, which is only set while a
    /// view renders. The same note `elements::button`'s and
    /// `elements::sidebar`'s harnesses carry.
    struct Harness {
        build: Box<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            (self.build)(window, cx)
        }
    }

    fn draw(
        cx: &mut TestAppContext,
        build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> &'static mut VisualTestContext {
        let window = cx.open_window(size(px(600.), px(400.)), move |_window, _cx| Harness {
            build: Box::new(build),
        });
        let cx = VisualTestContext::from_window(*std::ops::Deref::deref(&window), cx).into_mut();
        cx.run_until_parked();
        cx
    }

    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(selector)
            .unwrap_or_else(|| panic!("`{selector}` was never laid out"))
    }

    fn click(cx: &mut VisualTestContext, selector: &'static str) {
        let target = bounds(cx, selector).center();
        cx.simulate_click(target, Modifiers::default());
        cx.run_until_parked();
    }

    /// The state every read API has to survive: nothing open.
    #[test]
    fn an_empty_stack_reads_as_nothing() {
        assert_eq!(current(), None);
        assert!(!disabled_here(false));
        assert!(disabled_here(true), "a control's own flag still wins");
        assert_eq!(name_here(), None);
        assert!(focus_handle_here().is_none());
    }

    #[test]
    fn a_scope_is_visible_inside_it_and_not_after_it() {
        scope(FormContext::new().disabled(true).name("Street"), || {
            assert!(disabled_here(false));
            assert_eq!(name_here().as_deref(), Some("Street"));
        });

        assert!(!disabled_here(false));
        assert_eq!(name_here(), None);
    }

    /// The rule that makes "the group is disabled" mean anything.
    #[test]
    fn nesting_cannot_re_enable() {
        scope(FormContext::new().disabled(true), || {
            scope(FormContext::new().disabled(false), || {
                assert!(
                    disabled_here(false),
                    "an inner scope saying nothing about disabled must not re-enable"
                );
            });
        });
    }

    #[test]
    fn a_name_is_inherited_until_something_nearer_answers() {
        scope(FormContext::new().name("outer"), || {
            scope(FormContext::new(), || {
                assert_eq!(name_here().as_deref(), Some("outer"));
            });
            scope(FormContext::new().name("inner"), || {
                assert_eq!(name_here().as_deref(), Some("inner"));
            });
        });
    }

    /// A child that panics mid-layout must not leave the stack one deep — the
    /// failure that would make an unrelated control announce itself disabled.
    #[test]
    fn the_guard_closes_the_scope_through_a_panic() {
        let panicked = std::panic::catch_unwind(|| {
            scope(FormContext::new().disabled(true), || {
                panic!("a child blew up mid-layout");
            })
        });

        assert!(panicked.is_err());
        assert_eq!(current(), None, "the guard popped the scope anyway");
        assert!(!disabled_here(false));
    }

    #[gpui::test]
    fn a_fieldset_announces_its_legend_as_a_group(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| {
            announced(
                fieldset("billing").legend("Billing address"),
                window,
                cx,
            )
        });

        assert_eq!(announced.role, Some(Role::Group));
        assert_eq!(announced.name(), Some("Billing address"));
        assert_eq!(announced.id, Some(ElementId::Name("billing".into())));
    }

    /// `Role::Group` is read rather than operated, so a legend-less fieldset is
    /// a nameless node rather than a `debug_assert!`.
    #[gpui::test]
    fn a_fieldset_without_a_legend_still_announces(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| announced(fieldset("billing"), window, cx));

        assert_eq!(announced.role, Some(Role::Group));
        assert_eq!(announced.name(), None);
    }

    // ============================================================
    // END TO END
    // ============================================================
    //
    // Everything above proves the stack. These two draw a real window with a
    // real `Checkbox` entity in it, because that is the only way to see the
    // three read APIs actually land: the scope is opened by
    // `WithFormContext` during layout and paint, and no unit test over a
    // `Vec` can tell you whether the element tree ever entered it.

    /// A checkbox inside a disabled fieldset, where **nothing between them
    /// says `disabled`** — not the fieldset's field, not the checkbox.
    ///
    /// With no `aria_disabled` in gpui, the click action a node does not offer
    /// is what tells the two apart; that is the same test `Button`'s
    /// `a_disabled_button_offers_no_click_action` runs, one layer further out.
    #[gpui::test]
    fn a_group_disables_a_checkbox_that_says_nothing_about_it(cx: &mut TestAppContext) {
        cx.update(crate::init);

        let (enabled, disabled) = cx.update(|cx| {
            (
                cx.new(|_cx| checkbox("consent-open", false).label("Consent")),
                cx.new(|_cx| checkbox("consent-locked", false).label("Consent")),
            )
        });
        let (loose, grouped) = (enabled.clone(), disabled.clone());

        let cx = draw(cx, move |_window, _cx| {
            v_stack()
                .child(
                    fieldset("open")
                        .legend("Open")
                        .child(field("consent-open").label("Consent").child(loose.clone())),
                )
                .child(
                    fieldset("locked")
                        .legend("Locked")
                        .disabled(true)
                        .child(field("consent-locked").label("Consent").child(grouped.clone())),
                )
                .into_any_element()
        });

        let hit = |handle: &gpui::Entity<Checkbox>, cx: &mut VisualTestContext| {
            cx.update(|_window, cx| handle.read(cx).is_checked())
        };

        // The enabled one toggles.
        click(cx, r#"gpuikit-checkbox-Name("consent-open")"#);
        assert!(
            hit(&enabled, cx),
            "the checkbox outside the disabled group still toggles"
        );

        // The one in the disabled group does not, and it was never told.
        click(cx, r#"gpuikit-checkbox-Name("consent-locked")"#);
        assert!(
            !hit(&disabled, cx),
            "a checkbox inside a disabled fieldset must not toggle, even though neither it \
             nor its field says `disabled`"
        );
    }

    /// The other half of label association: a click on a `Field`'s label lands
    /// focus on the control beside it, through the handle the field published
    /// and the checkbox tracked.
    #[gpui::test]
    fn a_click_on_a_field_label_focuses_the_control_it_names(cx: &mut TestAppContext) {
        cx.update(crate::init);

        let control = cx.update(|cx| cx.new(|_cx| checkbox("consent", false).label("Consent")));
        let drawn = control.clone();

        let cx = draw(cx, move |_window, _cx| {
            fieldset("billing")
                .legend("Billing address")
                .child(field("street").label("Street").child(drawn.clone()))
                .into_any_element()
        });

        assert!(
            cx.update(|window, cx| window.focused(cx)).is_none(),
            "nothing is focused before the click"
        );

        click(cx, "gpuikit-field-label");

        let focused = cx.update(|window, cx| window.focused(cx));
        let expected = cx.update(|_window, cx| {
            field_focus_handle(&ElementId::Name("street".into()), cx)
        });
        assert_eq!(
            focused,
            Some(expected),
            "the label click focused the handle the field published and the checkbox tracked"
        );
        assert!(
            !control.read_with(cx, |checkbox, _| checkbox.is_checked()),
            "focusing a control is not operating it"
        );
    }

    /// Two calls for the same id hand back the same handle, which is what
    /// makes a label click land on the control the last frame drew.
    #[gpui::test]
    fn a_field_focus_handle_is_stable_per_id(cx: &mut TestAppContext) {
        let (first, second, other) = cx.update(|cx| {
            let id = ElementId::Name("street".into());
            let other = ElementId::Name("city".into());
            (
                field_focus_handle(&id, cx),
                field_focus_handle(&id, cx),
                field_focus_handle(&other, cx),
            )
        });

        assert_eq!(first, second);
        assert_ne!(first, other);
    }
}
