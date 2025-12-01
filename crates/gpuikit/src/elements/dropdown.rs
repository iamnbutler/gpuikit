//! Dropdown component for gpuikit

use crate::layout::{h_stack, v_stack};
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    anchored, deferred, div, prelude::*, rems, App, Context, DismissEvent, ElementId, Entity,
    EventEmitter, FocusHandle, Focusable, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};
use gpuikit_theme::ActiveTheme;
use std::rc::Rc;

/// Event emitted when the dropdown selection changes
pub struct DropdownChanged;

/// A single option in a dropdown menu
#[derive(Clone)]
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

/// The popup menu portion of a dropdown
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
        let theme = cx.theme();
        let focus_handle = self.focus_handle.clone();

        v_stack()
            .id("dropdown-menu")
            .track_focus(&focus_handle)
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .min_w(rems(7.5))
            .max_h(rems(20.0))
            .overflow_y_scroll()
            .on_scroll_wheel(|_, _, cx| {
                cx.stop_propagation();
            })
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(rems(0.25))
            .shadow_lg()
            .py(rems(0.25))
            .children(self.options.iter().enumerate().map(|(index, option)| {
                let is_selected = index == self.selected_index;
                let label = option.label.clone();
                let theme = theme.clone();

                div()
                    .id(ElementId::NamedInteger(
                        "dropdown-option".into(),
                        index as u64,
                    ))
                    .px(rems(0.5))
                    .py(rems(0.25))
                    .text_sm()
                    .cursor_pointer()
                    .when(is_selected, |this| {
                        this.bg(theme.accent_bg).text_color(theme.accent)
                    })
                    .when(!is_selected, |this| this.text_color(theme.fg))
                    .hover(|style| style.bg(theme.button_bg_hover))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select(index, window, cx);
                    }))
                    .child(label)
            }))
    }
}

/// Builder for creating a Dropdown component
pub struct Dropdown<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<(T, SharedString)>,
    selected: T,
    on_change: Option<Rc<dyn Fn(T, &mut Window, &mut App)>>,
    full_width: bool,
    disabled: bool,
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

    pub fn on_change(mut self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Stateful wrapper for Dropdown that manages the menu visibility
pub struct DropdownState<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<(T, SharedString)>,
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

    pub fn set_selected(&mut self, value: T, cx: &mut Context<Self>) {
        if self.selected != value {
            self.selected = value;
            cx.emit(DropdownChanged);
            cx.notify();
        }
    }

    fn selected_label(&self) -> SharedString {
        self.options
            .iter()
            .find(|(v, _)| *v == self.selected)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Select...".into())
    }

    fn selected_index(&self) -> usize {
        self.options
            .iter()
            .position(|(v, _)| *v == self.selected)
            .unwrap_or(0)
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
}

impl<T: Clone + PartialEq + 'static> Render for DropdownState<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_open = self.menu.is_some();
        let label = self.selected_label();
        let full_width = self.full_width;
        let disabled = self.disabled;

        div()
            .relative()
            .when(full_width, |this| this.w_full())
            .child(
                h_stack()
                    .id(self.id.clone())
                    .items_center()
                    .justify_between()
                    .gap(rems(0.5))
                    .px(rems(0.5))
                    .h(rems(1.5))
                    .min_w(rems(6.0))
                    .when(full_width, |this| this.w_full())
                    .bg(theme.input_bg)
                    .border_1()
                    .border_color(if is_open {
                        theme.input_border_focused
                    } else {
                        theme.input_border
                    })
                    .rounded(rems(0.25))
                    .text_sm()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(if disabled {
                        theme.fg_disabled
                    } else {
                        theme.fg
                    })
                    .when(!disabled, |this| {
                        this.cursor_pointer()
                            .hover(|style| style.border_color(theme.input_border_hover))
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.prevent_default()
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_menu(window, cx);
                            }))
                    })
                    .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
                    .child(label)
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.fg_muted)
                            .child(if is_open { "▲" } else { "▼" }),
                    ),
            )
            .when_some(self.menu.clone(), |this, menu| {
                this.child(
                    deferred(anchored().child(div().occlude().mt(rems(0.25)).child(menu)))
                        .with_priority(1),
                )
            })
    }
}

/// Convenience function to create a dropdown builder
pub fn dropdown<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<(T, impl Into<SharedString>)>,
    selected: T,
) -> Dropdown<T> {
    Dropdown::new(id, options, selected)
}

/// Convenience function to create a dropdown with auto-generated ID
pub fn dropdown_auto<T: Clone + PartialEq + 'static>(
    cx: &App,
    options: Vec<(T, impl Into<SharedString>)>,
    selected: T,
) -> Dropdown<T> {
    Dropdown::new(cx.next_id_named("dropdown"), options, selected)
}
