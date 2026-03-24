//! Context Menu component for right-click actions
//!
//! Provides a context menu triggered by right-click, displaying a positioned menu of actions.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::context_menu::{
//!     context_menu, context_menu_trigger, menu_item, menu_separator, ContextMenuState
//! };
//! use gpuikit::DefaultIcons;
//!
//! // Create the context menu state
//! let menu_state = cx.new(|_cx| ContextMenuState::new(
//!     context_menu("file-actions")
//!         .menu(|_window, _cx| {
//!             vec![
//!                 menu_item("copy", "Copy")
//!                     .icon(|| DefaultIcons::copy())
//!                     .kbd("Cmd+C"),
//!                 menu_item("paste", "Paste")
//!                     .icon(|| DefaultIcons::clipboard())
//!                     .kbd("Cmd+V"),
//!                 menu_separator(),
//!                 menu_item("delete", "Delete")
//!                     .icon(|| DefaultIcons::trash())
//!                     .destructive(),
//!             ]
//!         })
//!         .on_action(|action_id, _window, _cx| {
//!             println!("Selected: {}", action_id);
//!         })
//! ));
//!
//! // Use in render - wrap the target element with context_menu_trigger
//! context_menu_trigger(&menu_state, |_window, _cx| {
//!     div().p_4().child("Right-click me")
//! })
//! ```

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    actions, deferred, div, prelude::*, px, AnyElement, App, Context, DismissEvent, ElementId,
    Entity, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, MouseButton,
    ParentElement, Pixels, Point, Render, SharedString, Styled, Svg, Window,
};
use std::rc::Rc;

actions!(context_menu, [Close]);

/// The key context used for context menu keybindings.
pub const CONTEXT_MENU_CONTEXT: &str = "ContextMenu";

/// Event emitted when a menu item is selected.
pub struct ContextMenuAction {
    /// The id of the selected action.
    pub action_id: SharedString,
}

/// Represents a single item in a context menu.
#[derive(Clone)]
pub enum ContextMenuItem {
    /// A clickable action item.
    Action(MenuAction),
    /// A visual separator between groups of items.
    Separator,
}

/// A clickable action in a context menu.
pub struct MenuAction {
    id: SharedString,
    label: SharedString,
    icon: Option<Rc<dyn Fn() -> Svg>>,
    kbd: Option<SharedString>,
    disabled: bool,
    destructive: bool,
}

impl Clone for MenuAction {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            label: self.label.clone(),
            icon: self.icon.clone(),
            kbd: self.kbd.clone(),
            disabled: self.disabled,
            destructive: self.destructive,
        }
    }
}

impl MenuAction {
    /// Create a new menu action.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            kbd: None,
            disabled: false,
            destructive: false,
        }
    }

    /// Set an icon for this menu item.
    pub fn icon(mut self, icon: impl Fn() -> Svg + 'static) -> Self {
        self.icon = Some(Rc::new(icon));
        self
    }

    /// Set a keyboard shortcut hint for this menu item.
    pub fn kbd(mut self, kbd: impl Into<SharedString>) -> Self {
        self.kbd = Some(kbd.into());
        self
    }

    /// Mark this action as destructive (will be styled in danger color).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

impl Disableable for MenuAction {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Convenience function to create a menu action item.
pub fn menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> ContextMenuItem {
    ContextMenuItem::Action(MenuAction::new(id, label))
}

/// Convenience function to create a menu separator.
pub fn menu_separator() -> ContextMenuItem {
    ContextMenuItem::Separator
}

impl ContextMenuItem {
    /// Set an icon for this menu item (only applies to actions).
    pub fn icon(self, icon: impl Fn() -> Svg + 'static) -> Self {
        match self {
            ContextMenuItem::Action(action) => ContextMenuItem::Action(action.icon(icon)),
            ContextMenuItem::Separator => self,
        }
    }

    /// Set a keyboard shortcut hint (only applies to actions).
    pub fn kbd(self, kbd: impl Into<SharedString>) -> Self {
        match self {
            ContextMenuItem::Action(action) => ContextMenuItem::Action(action.kbd(kbd)),
            ContextMenuItem::Separator => self,
        }
    }

    /// Mark this item as destructive (only applies to actions).
    pub fn destructive(self) -> Self {
        match self {
            ContextMenuItem::Action(action) => ContextMenuItem::Action(action.destructive()),
            ContextMenuItem::Separator => self,
        }
    }
}

impl Disableable for ContextMenuItem {
    fn is_disabled(&self) -> bool {
        match self {
            ContextMenuItem::Action(action) => action.is_disabled(),
            ContextMenuItem::Separator => false,
        }
    }

    fn disabled(self, disabled: bool) -> Self {
        match self {
            ContextMenuItem::Action(action) => ContextMenuItem::Action(action.disabled(disabled)),
            ContextMenuItem::Separator => self,
        }
    }
}

/// The popup menu that displays context menu items.
pub struct ContextMenuPopup {
    items: Vec<ContextMenuItem>,
    focus_handle: FocusHandle,
    focused_index: Option<usize>,
    position: Point<Pixels>,
    on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

impl EventEmitter<DismissEvent> for ContextMenuPopup {}

impl Focusable for ContextMenuPopup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ContextMenuPopup {
    pub fn build(
        items: Vec<ContextMenuItem>,
        position: Point<Pixels>,
        on_action: impl Fn(SharedString, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                items,
                focus_handle,
                focused_index: None,
                position,
                on_action: Some(Rc::new(on_action)),
            }
        })
    }

    fn action_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| match item {
                ContextMenuItem::Action(action) if !action.disabled => Some(i),
                _ => None,
            })
            .collect()
    }

    fn focus_next(&mut self, cx: &mut Context<Self>) {
        let indices = self.action_indices();
        if indices.is_empty() {
            return;
        }

        let next = match self.focused_index {
            None => indices.first().copied(),
            Some(current) => {
                let pos = indices.iter().position(|&i| i == current).unwrap_or(0);
                indices.get(pos + 1).or(indices.first()).copied()
            }
        };

        self.focused_index = next;
        cx.notify();
    }

    fn focus_prev(&mut self, cx: &mut Context<Self>) {
        let indices = self.action_indices();
        if indices.is_empty() {
            return;
        }

        let prev = match self.focused_index {
            None => indices.last().copied(),
            Some(current) => {
                let pos = indices.iter().position(|&i| i == current).unwrap_or(0);
                if pos == 0 {
                    indices.last().copied()
                } else {
                    indices.get(pos - 1).copied()
                }
            }
        };

        self.focused_index = prev;
        cx.notify();
    }

    fn select_focused(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.focused_index {
            if let Some(ContextMenuItem::Action(action)) = self.items.get(index) {
                if !action.disabled {
                    self.select_action(action.id.clone(), window, cx);
                }
            }
        }
    }

    fn select_action(
        &mut self,
        action_id: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(on_action) = &self.on_action {
            let on_action = on_action.clone();
            on_action(action_id, window, cx);
        }
        cx.emit(DismissEvent);
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn handle_close(&mut self, _: &Close, _window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss(_window, cx);
    }
}

impl Render for ContextMenuPopup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();
        let position = self.position;

        div()
            .id("context-menu-popup")
            .key_context(CONTEXT_MENU_CONTEXT)
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::handle_close))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "down" => {
                        this.focus_next(cx);
                        cx.stop_propagation();
                    }
                    "up" => {
                        this.focus_prev(cx);
                        cx.stop_propagation();
                    }
                    "enter" | "space" => {
                        this.select_focused(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .absolute()
            .left(position.x)
            .top(position.y)
            .min_w(px(160.))
            .max_h(px(400.))
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
            .children(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| self.render_item(index, item, cx)),
            )
    }
}

impl ContextMenuPopup {
    fn render_item(
        &self,
        index: usize,
        item: &ContextMenuItem,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();

        match item {
            ContextMenuItem::Separator => div()
                .my_1()
                .mx_2()
                .h(px(1.))
                .bg(theme.border_subtle())
                .into_any_element(),

            ContextMenuItem::Action(action) => {
                let is_focused = self.focused_index == Some(index);
                let is_disabled = action.disabled;
                let is_destructive = action.destructive;
                let action_id = action.id.clone();
                let label = action.label.clone();
                let icon_fn = action.icon.clone();
                let kbd = action.kbd.clone();

                let text_color = if is_disabled {
                    theme.fg_disabled()
                } else if is_destructive {
                    theme.danger()
                } else {
                    theme.fg()
                };

                let bg_color = if is_focused && !is_disabled {
                    theme.surface_secondary()
                } else {
                    gpui::transparent_black()
                };

                let icon_color = if is_disabled {
                    theme.fg_disabled()
                } else if is_destructive {
                    theme.danger()
                } else {
                    theme.fg_muted()
                };

                div()
                    .id(ElementId::NamedInteger("context-menu-item".into(), index as u64))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .px_3()
                    .py_1p5()
                    .text_xs()
                    .bg(bg_color)
                    .text_color(text_color)
                    .when(!is_disabled, |this| {
                        this.cursor_pointer()
                            .hover(|style| style.bg(theme.surface_secondary()))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select_action(action_id.clone(), window, cx);
                            }))
                    })
                    .when(is_disabled, |this| this.cursor_not_allowed())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(icon_fn, |this, icon_fn| {
                                this.child(icon_fn().size(px(14.)).text_color(icon_color))
                            })
                            .child(label),
                    )
                    .when_some(kbd, |this, kbd| {
                        this.child(
                            div()
                                .text_color(theme.fg_muted())
                                .text_size(px(10.))
                                .child(kbd),
                        )
                    })
                    .into_any_element()
            }
        }
    }
}

/// Builder for creating a context menu component.
pub struct ContextMenu {
    id: ElementId,
    menu: Option<Rc<dyn Fn(&mut Window, &mut App) -> Vec<ContextMenuItem>>>,
    on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

/// Creates a new context menu builder.
pub fn context_menu(id: impl Into<ElementId>) -> ContextMenu {
    ContextMenu::new(id)
}

impl ContextMenu {
    /// Create a new context menu builder.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            menu: None,
            on_action: None,
        }
    }

    /// Set the menu items builder function.
    pub fn menu(
        mut self,
        menu: impl Fn(&mut Window, &mut App) -> Vec<ContextMenuItem> + 'static,
    ) -> Self {
        self.menu = Some(Rc::new(menu));
        self
    }

    /// Register a callback for when an action is selected.
    pub fn on_action(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Rc::new(handler));
        self
    }
}

/// Stateful context menu component that manages the popup lifecycle.
pub struct ContextMenuState {
    id: ElementId,
    menu: Option<Rc<dyn Fn(&mut Window, &mut App) -> Vec<ContextMenuItem>>>,
    popup: Option<Entity<ContextMenuPopup>>,
    on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

impl EventEmitter<ContextMenuAction> for ContextMenuState {}

impl ContextMenuState {
    /// Create a new context menu state from a builder.
    pub fn new(menu: ContextMenu) -> Self {
        Self {
            id: menu.id,
            menu: menu.menu,
            popup: None,
            on_action: menu.on_action,
        }
    }

    /// Check if the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.popup.is_some()
    }

    /// Open the menu at the given position (in pixels).
    pub fn open(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        // Close any existing popup first
        self.popup = None;

        let items = if let Some(menu_fn) = &self.menu {
            menu_fn(window, cx)
        } else {
            return;
        };

        if items.is_empty() {
            return;
        }

        let on_action = self.on_action.clone();
        let entity = cx.entity().downgrade();

        let popup = ContextMenuPopup::build(
            items,
            position,
            move |action_id, window, cx| {
                if let Some(on_action) = &on_action {
                    on_action(action_id.clone(), window, cx);
                }
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |_, cx| {
                        cx.emit(ContextMenuAction { action_id });
                    });
                }
            },
            window,
            cx,
        );

        cx.subscribe_in(
            &popup,
            window,
            |this, _, _event: &DismissEvent, _window, cx| {
                this.popup = None;
                cx.notify();
            },
        )
        .detach();

        self.popup = Some(popup);
        cx.notify();
    }

    /// Close the menu.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.popup = None;
        cx.notify();
    }

    /// Get the element ID.
    pub fn id(&self) -> &ElementId {
        &self.id
    }
}

impl Render for ContextMenuState {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Just render the popup overlay if open
        div().when_some(self.popup.clone(), |this, popup| {
            this.child(deferred(div().absolute().size_full().child(popup)).with_priority(5))
        })
    }
}

/// Creates a context menu trigger element.
///
/// Wraps a child element and opens the menu on right-click.
///
/// # Example
///
/// ```ignore
/// context_menu_trigger(&menu_state, |_window, _cx| {
///     div().p_4().child("Right-click me")
/// })
/// ```
pub fn context_menu_trigger<E: IntoElement + 'static>(
    state: &Entity<ContextMenuState>,
    child: impl Fn(&mut Window, &mut App) -> E + 'static,
) -> ContextMenuTrigger<E> {
    ContextMenuTrigger {
        state: state.clone(),
        child: Box::new(child),
    }
}

/// A trigger element for opening a context menu on right-click.
pub struct ContextMenuTrigger<E: IntoElement> {
    state: Entity<ContextMenuState>,
    child: Box<dyn Fn(&mut Window, &mut App) -> E>,
}

impl<E: IntoElement + 'static> IntoElement for ContextMenuTrigger<E> {
    type Element = gpui::Stateful<gpui::Div>;

    fn into_element(self) -> Self::Element {
        let state = self.state.clone();

        div()
            .id("context-menu-trigger")
            .on_mouse_down(
                MouseButton::Right,
                move |event, window: &mut Window, cx: &mut App| {
                    let position = event.position;
                    state.update(cx, |menu, cx| {
                        menu.open(position, window, cx);
                    });
                    cx.stop_propagation();
                },
            )
            .into_element()
    }
}

impl<E: IntoElement + 'static> RenderOnce for ContextMenuTrigger<E> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.clone();
        let state_for_closure = state.clone();
        let child = (self.child)(window, cx);

        div()
            .id("context-menu-trigger")
            .child(child)
            .on_mouse_down(
                MouseButton::Right,
                move |event, window: &mut Window, cx: &mut App| {
                    let position = event.position;
                    state_for_closure.update(cx, |menu, cx| {
                        menu.open(position, window, cx);
                    });
                    cx.stop_propagation();
                },
            )
            .child(state)
    }
}

/// Binds the context menu keybindings to the application.
///
/// Call this in your application's initialization to enable escape-to-close functionality.
pub fn bind_context_menu_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "escape",
        Close,
        Some(CONTEXT_MENU_CONTEXT),
    )]);
}
