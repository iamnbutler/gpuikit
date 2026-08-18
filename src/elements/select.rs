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
//! Two things this element still cannot say. It has **no keyboard model in the
//! popup** — no arrow keys, no Escape, no roving focus between options. The
//! trigger is a tab stop and Enter or Space opens it, which is what
//! `.focusable()` and gpui's keyboard activation give it; everything after that
//! is still the mouse's. And it reports **no `aria-activedescendant`**: that
//! property names the row keyboard focus is virtually on, and until there is a
//! keyboard model there is no such row. Both arrive together.

use crate::a11y::{A11y, Announce};
use crate::element_id::for_entity;
use crate::theme::{focus_ring, ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    anchored, deferred, div, point, prelude::*, px, App, Context, DismissEvent, ElementId, Entity,
    EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Rems, Render, Role,
    SharedString, Styled, Window,
};

use crate::icons::Icons;
use std::rc::Rc;

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
    /// The rung of the trigger that opened this listbox, so a popup's rows are
    /// the same size as the control they dropped out of.
    size: ControlSize,
    focus_handle: FocusHandle,
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
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                label,
                options,
                selected_index,
                size,
                focus_handle,
                on_select: Some(Rc::new(on_select)),
            }
        })
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_select) = &self.on_select {
            let on_select = on_select.clone();
            on_select(index, window, cx);
        }
        cx.emit(DismissEvent);
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
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
fn option_a11y(label: SharedString, is_selected: bool, index: usize, count: usize) -> A11y {
    A11y::new(Role::ListBoxOption)
        .name(label)
        .selected(is_selected)
        // Both, together: a position with no size announces "3" out of nowhere.
        .position_in_set(index + 1)
        .size_of_set(count)
}

impl Render for Listbox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let count = self.options.len();
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        div()
            // Unique only because a `Listbox` is always rendered as an
            // `Entity<_>`, which puts an `ElementId::View` above it.
            .id(for_entity("select-listbox", cx.entity_id()))
            .announce(self.a11y())
            // Focus stays declared here rather than through `A11y`: the popup
            // is focused programmatically when it opens, and making it a tab
            // stop would put a transient overlay in the tab order. It becomes
            // an `A11y::focus_handle` when the listbox grows a keyboard model.
            .track_focus(&focus_handle)
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .min_w(px(120.))
            .max_h(px(480.))
            .overflow_y_scroll()
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
                let label = label.clone();
                let theme = cx.theme();
                let a11y = option_a11y(label.clone(), is_selected, index, count);

                div()
                    .id(ElementId::NamedInteger(
                        "select-option".into(),
                        index as u64,
                    ))
                    .announce(a11y)
                    .flex()
                    .items_center()
                    .h(metrics.height)
                    .px(metrics.padding_x * 1.5)
                    .text_size(metrics.text_size)
                    .line_height(metrics.line_height)
                    .cursor_pointer()
                    .when(is_selected, |this| {
                        this.bg(theme.accent()).text_color(theme.bg())
                    })
                    .when(!is_selected, |this| {
                        this.text_color(theme.fg())
                            .hover(|style| style.bg(theme.surface_secondary()))
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select(index, window, cx);
                    }))
                    .child(label)
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
#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, size, Bounds, Pixels, Render, TestAppContext, VisualTestContext};
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
        cx.update(crate::theme::init);

        let window = cx.open_window(window_size, |_window, cx| {
            let select = cx.new(|_cx| {
                let options: Vec<(usize, SharedString)> = (0..options)
                    .map(|index| (index, SharedString::from(format!("Option {index}"))))
                    .collect();
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
        let node = node_for(option_a11y("Option 1".into(), true, 1, 3));

        assert_eq!(node.label(), Some("Option 1"));
        assert_eq!(node.is_selected(), Some(true));
        assert_eq!(node.position_in_set(), Some(2), "counted from 1");
        assert_eq!(node.size_of_set(), Some(3));

        let unchosen = node_for(option_a11y("Option 2".into(), false, 2, 3));
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
