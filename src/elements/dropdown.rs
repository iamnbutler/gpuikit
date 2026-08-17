//! Dropdown
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::traits::disableable::Disableable;
//!
//! // Create a dropdown with enum options
//! #[derive(Clone, PartialEq)]
//! enum Size { Small, Medium, Large }
//!
//! let dropdown_state = cx.new(|_cx| {
//!     DropdownState::new(
//!         dropdown(
//!             "size-dropdown",
//!             vec![
//!                 (Size::Small, "Small"),
//!                 (Size::Medium, "Medium"),
//!                 (Size::Large, "Large"),
//!             ],
//!             Size::Medium,
//!         )
//!         .on_change(|value, _window, _cx| {
//!             println!("Selected: {:?}", value);
//!         })
//!         .disabled(false) // Set to true to disable the dropdown
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

/// The gap between a trigger and the popup that drops out of it.
///
/// Applied through `anchored().offset(…)` and never as a margin on the
/// anchored child: `Anchored::prepaint` fits the *union of its children's
/// layout bounds* to the window, and a margin sits outside that union, so a
/// popup at the bottom of the window would be clamped correctly and then
/// pushed straight back out by its own margin. See `docs/overlays.md`.
///
/// Shared with `select.rs`, which drops the same `DropdownMenu` out of the
/// same trigger shape.
pub(crate) const MENU_GAP: Rems = Rems(0.25);

/// Event emitted when the dropdown selection changes.
pub struct DropdownChanged;

/// A single option in the dropdown menu.
pub struct DropdownOption {
    pub label: SharedString,
}

impl DropdownOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// The popup menu that displays dropdown options.
pub struct DropdownMenu {
    options: Vec<DropdownOption>,
    selected_index: usize,
    /// The rung of the trigger that opened this menu, so a popup's rows are
    /// the same size as the control they dropped out of.
    size: ControlSize,
    focus_handle: FocusHandle,
    on_select: Option<Rc<dyn Fn(usize, &mut Window, &mut App)>>,
}

impl EventEmitter<DismissEvent> for DropdownMenu {}

impl Focusable for DropdownMenu {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DropdownMenu {
    pub fn build(
        options: Vec<DropdownOption>,
        selected_index: usize,
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

impl Render for DropdownMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        div()
            // Was unique only because a `DropdownMenu` is always rendered as an
            // `Entity<_>`, which puts an `ElementId::View` above it.
            .id(for_entity("dropdown-menu", cx.entity_id()))
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
            .children(self.options.iter().enumerate().map(|(index, option)| {
                let is_selected = index == self.selected_index;
                let label = option.label.clone();
                let theme = cx.theme();

                div()
                    .id(ElementId::NamedInteger(
                        "dropdown-option".into(),
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

/// Builder for creating a dropdown component.
///
/// Use the [`dropdown`] function to create an instance.
pub struct Dropdown<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<(T, SharedString)>,
    selected: T,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

/// Creates a new dropdown builder.
///
/// # Arguments
///
/// * `id` - Unique identifier for the dropdown
/// * `options` - Vector of (value, label) tuples
/// * `selected` - The currently selected value
///
/// # Example
///
/// ```ignore
/// dropdown(
///     "my-dropdown",
///     vec![("a", "Option A"), ("b", "Option B")],
///     "a",
/// )
/// ```
pub fn dropdown<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<(T, impl Into<SharedString>)>,
    selected: T,
) -> Dropdown<T> {
    Dropdown::new(id, options, selected)
}

impl<T: Clone + PartialEq + 'static> Dropdown<T> {
    pub fn new(
        id: impl Into<ElementId>,
        options: Vec<(T, impl Into<SharedString>)>,
        selected: T,
    ) -> Self {
        Self {
            id: id.into(),
            options: options
                .into_iter()
                .map(|(value, label)| (value, label.into()))
                .collect(),
            selected,
            on_change: None,
            full_width: false,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    /// Register a callback for when the selection changes.
    pub fn on_change(mut self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Make the dropdown expand to fill available width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}

impl<T: Clone + PartialEq + 'static> Disableable for Dropdown<T> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Clone + PartialEq + 'static> ControlSized for Dropdown<T> {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

/// Stateful dropdown component that manages the menu popup.
///
/// Create using [`Dropdown`] and wrap in an Entity:
///
/// ```ignore
/// let state = cx.new(|_cx| DropdownState::new(dropdown(...)));
/// ```
pub struct DropdownState<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<(T, SharedString)>,
    /// The currently selected value.
    pub selected: T,
    menu: Option<Entity<DropdownMenu>>,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
    size: ControlSize,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<DropdownChanged> for DropdownState<T> {}

impl<T: Clone + PartialEq + 'static> DropdownState<T> {
    pub fn new(dropdown: Dropdown<T>) -> Self {
        Self {
            id: dropdown.id,
            options: dropdown.options,
            selected: dropdown.selected,
            menu: None,
            on_change: dropdown.on_change,
            full_width: dropdown.full_width,
            disabled: dropdown.disabled,
            size: dropdown.size,
        }
    }

    /// Get the label of the currently selected option.
    fn selected_label(&self) -> SharedString {
        self.options
            .iter()
            .find(|(v, _)| *v == self.selected)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Select...".into())
    }

    /// Get the index of the currently selected option.
    fn selected_index(&self) -> usize {
        self.options
            .iter()
            .position(|(v, _)| *v == self.selected)
            .unwrap_or(0)
    }

    /// Update the selected value programmatically.
    pub fn set_selected(&mut self, value: T, cx: &mut Context<Self>) {
        self.selected = value;
        cx.emit(DropdownChanged);
        cx.notify();
    }

    /// Check if the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.menu.is_some()
    }

    /// Check if the dropdown is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Set the disabled state programmatically.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        if disabled && self.menu.is_some() {
            self.menu = None;
        }
        cx.notify();
    }

    fn toggle_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }

        if self.menu.is_some() {
            self.menu = None;
            cx.notify();
            return;
        }

        let options: Vec<DropdownOption> = self
            .options
            .iter()
            .map(|(_, label)| DropdownOption::new(label.clone()))
            .collect();

        let selected_index = self.selected_index();
        let values: Vec<T> = self.options.iter().map(|(v, _)| v.clone()).collect();
        let on_change = self.on_change.clone();

        let entity = cx.entity().downgrade();
        let menu = DropdownMenu::build(
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
                            state.selected = value;
                            cx.emit(DropdownChanged);
                            cx.notify();
                        });
                    }
                }
            },
            window,
            cx,
        );

        cx.subscribe_in(
            &menu,
            window,
            |this, _, _event: &DismissEvent, _window, cx| {
                this.menu = None;
                cx.notify();
            },
        )
        .detach();

        self.menu = Some(menu);
        cx.notify();
    }

    pub fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.menu.is_some();
        let label = self.selected_label();
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
            .text_color(if disabled {
                theme.fg_disabled()
            } else {
                theme.fg()
            })
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(|style| style.border_color(theme.input_border_hover()))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_menu(window, cx);
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
        let trigger = trigger.debug_selector(|| "gpuikit-dropdown-trigger".into());

        // The gap goes on the anchored element, not on its child: gpui fits
        // the union of the child's *layout bounds* to the window, and a margin
        // is outside it. See `MENU_GAP`.
        let gap = MENU_GAP.to_pixels(window.rem_size());

        div()
            .relative()
            .when(full_width, |this| this.w_full())
            .child(trigger)
            .when_some(self.menu.clone(), |this, menu| {
                let popup = div().occlude().child(menu);

                #[cfg(test)]
                let popup = popup.debug_selector(|| "gpuikit-dropdown-popup".into());

                this.child(
                    deferred(anchored().offset(point(px(0.), gap)).child(popup)).with_priority(1),
                )
            })
    }
}

impl<T: Clone + PartialEq + 'static> Render for DropdownState<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render(window, cx)
    }
}

/// The one property `docs/overlays.md` claims and nothing else could check:
/// that a popup placed with `anchored()` really does stay inside the window,
/// *and* really does keep its gap from the trigger. Those two pulled against
/// each other while the gap was a margin on the anchored child — gpui fits the
/// union of the child's layout bounds to the window, and a margin is outside
/// it, so the popup was clamped correctly and then pushed back out by the gap.
#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, size, Bounds, Pixels, Render, TestAppContext, VisualTestContext};
    use std::ops::Deref;

    struct TestView {
        dropdown: Entity<DropdownState<usize>>,
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
                .child(self.dropdown.clone())
        }
    }

    /// Open a dropdown `top` pixels down a window of `window_size`, and report
    /// where the trigger and the popup were laid out.
    ///
    /// `options` is how many rows the menu has, which is how tall it is —
    /// enough of them and it fits neither below the trigger nor above it, so
    /// gpui refuses to flip and falls back to clamping into the window. That
    /// is the path the margin bug lived on.
    fn open_dropdown(
        cx: &mut TestAppContext,
        window_size: gpui::Size<Pixels>,
        top: Pixels,
        options: usize,
    ) -> (Bounds<Pixels>, Bounds<Pixels>) {
        cx.update(crate::theme::init);

        let window = cx.open_window(window_size, |_window, cx| {
            let dropdown = cx.new(|_cx| {
                let options: Vec<(usize, SharedString)> = (0..options)
                    .map(|index| (index, SharedString::from(format!("Option {index}"))))
                    .collect();
                DropdownState::new(dropdown("test-dropdown", options, 0usize))
            });
            TestView { dropdown, top }
        });

        let cx = VisualTestContext::from_window(*window.deref(), cx).into_mut();
        cx.run_until_parked();

        let trigger = cx
            .debug_bounds("gpuikit-dropdown-trigger")
            .expect("the trigger should have been laid out");
        cx.simulate_click(trigger.center(), gpui::Modifiers::default());
        cx.run_until_parked();

        let popup = cx
            .debug_bounds("gpuikit-dropdown-popup")
            .expect("the popup should have been laid out");

        (trigger, popup)
    }

    #[gpui::test]
    fn a_popup_opened_at_the_bottom_of_the_window_stays_inside_it(cx: &mut TestAppContext) {
        // Eight rows in a 240px window: the menu fits in the window but
        // neither below the trigger nor above it, so gpui declines to flip and
        // clamps into the window instead. That is the path the margin bug
        // lived on — the clamp was right and the margin then undid it.
        let window = size(px(320.), px(240.));
        let (_trigger, popup) = open_dropdown(cx, window, px(120.), 8);

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
        let (trigger, popup) = open_dropdown(cx, size(px(320.), px(800.)), px(40.), 3);

        let gap = popup.top() - trigger.bottom();
        // One rem at the test window's default rem size. Read from the
        // constant rather than restated, so the two cannot drift.
        let expected = MENU_GAP.to_pixels(px(16.));

        assert!(
            (gap - expected).abs() <= px(1.),
            "the popup hangs {gap:?} below the trigger, expected {expected:?}"
        );
    }
}
