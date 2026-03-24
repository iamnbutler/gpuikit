//! Select
//!
//! A form select component for choosing a single value from a list of options.
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

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    anchored, deferred, div, prelude::*, px, App, Context, DismissEvent, ElementId, Entity,
    EventEmitter, IntoElement, ParentElement, Render, SharedString, Styled, Window,
};

use crate::elements::dropdown::{DropdownMenu, DropdownOption};
use crate::icons::Icons;
use std::rc::Rc;

/// Event emitted when the select value changes.
pub struct SelectChanged;

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
        }
    }

    /// Set the initially selected value.
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
    menu: Option<Entity<DropdownMenu>>,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<SelectChanged> for SelectState<T> {}

impl<T: Clone + PartialEq + 'static> SelectState<T> {
    pub fn new(select: Select<T>) -> Self {
        Self {
            id: select.id,
            options: select.options,
            selected: select.selected,
            placeholder: select.placeholder,
            menu: None,
            on_change: select.on_change,
            full_width: select.full_width,
            disabled: select.disabled,
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

    /// Get the index of the currently selected option, or None if nothing selected.
    fn selected_index(&self) -> Option<usize> {
        self.selected.as_ref().and_then(|selected| {
            self.options.iter().position(|(v, _)| v == selected)
        })
    }

    /// Update the selected value programmatically.
    pub fn set_selected(&mut self, value: Option<T>, cx: &mut Context<Self>) {
        self.selected = value;
        cx.emit(SelectChanged);
        cx.notify();
    }

    /// Check if the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.menu.is_some()
    }

    /// Check if the select is disabled.
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

    /// Clear the selection.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.selected = None;
        cx.emit(SelectChanged);
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

        let selected_index = self.selected_index().unwrap_or(usize::MAX);
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
        let (label, is_placeholder) = self.display_label();
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

        let text_color = if disabled {
            theme.fg_disabled()
        } else if is_placeholder {
            theme.input_placeholder()
        } else {
            theme.fg()
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
                    .text_color(text_color)
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

impl<T: Clone + PartialEq + 'static> Render for SelectState<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render(window, cx)
    }
}
