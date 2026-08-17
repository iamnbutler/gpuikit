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

use crate::element_id::for_entity;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use gpui::{
    anchored, deferred, div, point, prelude::*, px, App, Context, DismissEvent, ElementId, Entity,
    EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Rems, Render, SharedString,
    Styled, Window,
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
    fn build(
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

impl Render for Listbox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        div()
            // Unique only because a `Listbox` is always rendered as an
            // `Entity<_>`, which puts an `ElementId::View` above it.
            .id(for_entity("select-listbox", cx.entity_id()))
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

                div()
                    .id(ElementId::NamedInteger(
                        "select-option".into(),
                        index as u64,
                    ))
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
/// * `options` - Vector of (value, label) tuples
///
/// # Example
///
/// ```ignore
/// select(
///     "my-select",
///     vec![("a", "Option A"), ("b", "Option B")],
/// )
/// .selected("a")
/// .placeholder("Choose an option...")
/// ```
pub fn select<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<(T, impl Into<SharedString>)>,
) -> Select<T> {
    Select::new(id, options)
}

impl<T: Clone + PartialEq + 'static> Select<T> {
    pub fn new(id: impl Into<ElementId>, options: Vec<(T, impl Into<SharedString>)>) -> Self {
        Self {
            id: id.into(),
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

    pub fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.listbox.is_some();
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
                let mut builder = super::select("test-select", options);
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
