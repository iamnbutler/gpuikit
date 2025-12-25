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

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    anchored, deferred, div, prelude::*, px, App, Context, DismissEvent, ElementId, Entity,
    EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, SharedString, Styled,
    Window,
};

use crate::icons::Icons;
use std::rc::Rc;

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
        on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            Self {
                options,
                selected_index,
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

        div()
            .id("dropdown-menu")
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
            .rounded_md()
            .shadow_lg()
            .py_1()
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
                    .px_3()
                    .py_1()
                    .text_xs()
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

    pub fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.menu.is_some();
        let label = self.selected_label();
        let full_width = self.full_width;
        let disabled = self.disabled;
        let theme = cx.theme();

        let border_color = if disabled {
            theme.border_subtle()
        } else if is_open {
            theme.input_border_focused()
        } else {
            theme.input_border()
        };

        div()
            .relative()
            .when(full_width, |this| this.w_full())
            .child(
                div()
                    .id(self.id.clone())
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .min_w(px(100.))
                    .when(full_width, |this| this.w_full())
                    .bg(theme.input_bg())
                    .border_1()
                    .border_color(border_color)
                    .rounded_sm()
                    .text_xs()
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
                                .size(px(12.))
                                .text_color(theme.fg_muted()),
                        ),
                    ),
            )
            .when_some(self.menu.clone(), |this, menu| {
                this.child(
                    deferred(anchored().child(div().occlude().mt_1().child(menu))).with_priority(1),
                )
            })
    }
}

impl<T: Clone + PartialEq + 'static> Render for DropdownState<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render(window, cx)
    }
}
