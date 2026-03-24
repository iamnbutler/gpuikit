//! Context Menu
//!
//! A context menu triggered by right-click, displaying a positioned menu of actions.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::context_menu::{context_menu, menu_item, menu_separator};
//!
//! let context_menu_state = cx.new(|_cx| {
//!     ContextMenuState::new(
//!         context_menu("file-actions")
//!             .trigger(|_window, _cx| {
//!                 div()
//!                     .p_4()
//!                     .bg(theme.surface())
//!                     .child("Right-click me")
//!                     .into_any_element()
//!             })
//!             .menu(|_window, _cx| {
//!                 vec![
//!                     menu_item("copy", "Copy").kbd("Cmd+C"),
//!                     menu_item("paste", "Paste").kbd("Cmd+V"),
//!                     menu_separator(),
//!                     menu_item("delete", "Delete").destructive(),
//!                 ]
//!             })
//!             .on_action(|id, _window, _cx| {
//!                 println!("Action: {}", id);
//!             })
//!     )
//! });
//! ```

use crate::elements::kbd::kbd;
use crate::elements::separator::separator;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    anchored, deferred, div, point, prelude::*, px, AnyElement, App, Context, DismissEvent,
    ElementId, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, Render, SharedString, Styled, Svg, Window,
};
use std::rc::Rc;

type IconBuilder = Rc<dyn Fn() -> Svg>;

/// A single item in the context menu.
pub struct MenuItem {
    id: SharedString,
    label: SharedString,
    icon: Option<IconBuilder>,
    kbd: Option<SharedString>,
    disabled: bool,
    destructive: bool,
}

impl MenuItem {
    /// Create a new menu item.
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
    pub fn kbd(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.kbd = Some(shortcut.into());
        self
    }

    /// Mark this menu item as disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Mark this menu item as destructive (will be styled in red).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

/// Creates a new menu item.
pub fn menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> MenuEntry {
    MenuEntry::Item(MenuItem::new(id, label))
}

/// Creates a menu separator.
pub fn menu_separator() -> MenuEntry {
    MenuEntry::Separator
}

/// An entry in the context menu (either an item or a separator).
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

impl MenuEntry {
    /// Set an icon for this menu entry (only applies to items).
    pub fn icon(self, icon: impl Fn() -> Svg + 'static) -> Self {
        match self {
            MenuEntry::Item(item) => MenuEntry::Item(item.icon(icon)),
            MenuEntry::Separator => self,
        }
    }

    /// Set a keyboard shortcut hint (only applies to items).
    pub fn kbd(self, shortcut: impl Into<SharedString>) -> Self {
        match self {
            MenuEntry::Item(item) => MenuEntry::Item(item.kbd(shortcut)),
            MenuEntry::Separator => self,
        }
    }

    /// Mark this entry as disabled (only applies to items).
    pub fn disabled(self, disabled: bool) -> Self {
        match self {
            MenuEntry::Item(item) => MenuEntry::Item(item.disabled(disabled)),
            MenuEntry::Separator => self,
        }
    }

    /// Mark this entry as destructive (only applies to items).
    pub fn destructive(self) -> Self {
        match self {
            MenuEntry::Item(item) => MenuEntry::Item(item.destructive()),
            MenuEntry::Separator => self,
        }
    }
}

type TriggerBuilder = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type MenuBuilder = Box<dyn Fn(&mut Window, &mut App) -> Vec<MenuEntry>>;

/// Internal representation of a menu item for rendering.
struct MenuItemData {
    id: SharedString,
    label: SharedString,
    icon: Option<IconBuilder>,
    kbd: Option<SharedString>,
    disabled: bool,
    destructive: bool,
}

/// Internal representation of a menu entry for rendering.
enum MenuEntryData {
    Item(MenuItemData),
    Separator,
}

impl From<MenuEntry> for MenuEntryData {
    fn from(entry: MenuEntry) -> Self {
        match entry {
            MenuEntry::Item(item) => MenuEntryData::Item(MenuItemData {
                id: item.id,
                label: item.label,
                icon: item.icon,
                kbd: item.kbd,
                disabled: item.disabled,
                destructive: item.destructive,
            }),
            MenuEntry::Separator => MenuEntryData::Separator,
        }
    }
}

/// The popup menu that displays context menu items.
pub struct ContextMenuPopup {
    entries: Vec<MenuEntryData>,
    focus_handle: FocusHandle,
    focused_index: Option<usize>,
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
        entries: Vec<MenuEntry>,
        on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                entries: entries.into_iter().map(Into::into).collect(),
                focus_handle,
                focused_index: None,
                on_action,
            }
        })
    }

    fn item_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                MenuEntryData::Item(item) if !item.disabled => Some(i),
                _ => None,
            })
            .collect()
    }

    fn select_action(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_action) = &self.on_action {
            let on_action = on_action.clone();
            on_action(id, window, cx);
        }
        cx.emit(DismissEvent);
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn move_focus(&mut self, delta: i32, _window: &mut Window, cx: &mut Context<Self>) {
        let indices = self.item_indices();
        if indices.is_empty() {
            return;
        }

        let new_focused = match self.focused_index {
            Some(current) => {
                let current_pos = indices.iter().position(|&i| i == current).unwrap_or(0);
                let new_pos = if delta > 0 {
                    (current_pos + 1) % indices.len()
                } else {
                    (current_pos + indices.len() - 1) % indices.len()
                };
                indices[new_pos]
            }
            None => {
                if delta > 0 {
                    indices[0]
                } else {
                    indices[indices.len() - 1]
                }
            }
        };

        self.focused_index = Some(new_focused);
        cx.notify();
    }

    fn activate_focused(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.focused_index {
            if let Some(MenuEntryData::Item(item)) = self.entries.get(index) {
                if !item.disabled {
                    let id = item.id.clone();
                    self.select_action(id, window, cx);
                }
            }
        }
    }
}

impl Render for ContextMenuPopup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();

        div()
            .id("context-menu-popup")
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => this.dismiss(window, cx),
                    "up" => this.move_focus(-1, window, cx),
                    "down" => this.move_focus(1, window, cx),
                    "enter" => this.activate_focused(window, cx),
                    _ => {}
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .min_w(px(160.))
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
            .children(self.entries.iter().enumerate().map(|(index, entry)| {
                let theme = cx.theme();
                let is_focused = self.focused_index == Some(index);

                match entry {
                    MenuEntryData::Item(item) => {
                        let id = item.id.clone();
                        let label = item.label.clone();
                        let icon = item.icon.as_ref().map(|f| f());
                        let kbd_text = item.kbd.clone();
                        let disabled = item.disabled;
                        let destructive = item.destructive;

                        let text_color = if disabled {
                            theme.fg_disabled()
                        } else if destructive {
                            theme.danger()
                        } else {
                            theme.fg()
                        };

                        div()
                            .id(ElementId::NamedInteger(
                                "context-menu-item".into(),
                                index as u64,
                            ))
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .px_3()
                            .py_1p5()
                            .text_xs()
                            .text_color(text_color)
                            .when(disabled, |this| this.cursor_not_allowed())
                            .when(!disabled, |this| {
                                this.cursor_pointer()
                                    .when(is_focused, |this| this.bg(theme.surface_secondary()))
                                    .hover(|style| style.bg(theme.surface_secondary()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.select_action(id.clone(), window, cx);
                                    }))
                                    .on_mouse_move(cx.listener(move |this, _, _, cx| {
                                        if this.focused_index != Some(index) {
                                            this.focused_index = Some(index);
                                            cx.notify();
                                        }
                                    }))
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .when_some(icon, |this, icon| {
                                        this.child(
                                            icon.size(px(14.)).text_color(if disabled {
                                                theme.fg_disabled()
                                            } else {
                                                theme.fg_muted()
                                            }),
                                        )
                                    })
                                    .child(label),
                            )
                            .when_some(kbd_text, |this, kbd_text| {
                                this.child(kbd(kbd_text).small())
                            })
                            .into_any_element()
                    }
                    MenuEntryData::Separator => div().my_1().child(separator()).into_any_element(),
                }
            }))
    }
}

/// Builder for creating a context menu component.
pub struct ContextMenu {
    id: ElementId,
    trigger: Option<TriggerBuilder>,
    menu: Option<MenuBuilder>,
    on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

/// Creates a new context menu builder.
pub fn context_menu(id: impl Into<ElementId>) -> ContextMenu {
    ContextMenu::new(id)
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            menu: None,
            on_action: None,
        }
    }

    /// Set the trigger element that will show the context menu on right-click.
    pub fn trigger(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(builder));
        self
    }

    /// Set the menu items builder function.
    pub fn menu(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> Vec<MenuEntry> + 'static,
    ) -> Self {
        self.menu = Some(Box::new(builder));
        self
    }

    /// Register a callback for when a menu item is selected.
    pub fn on_action(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Rc::new(handler));
        self
    }
}

/// Stateful context menu component that manages the popup.
pub struct ContextMenuState {
    id: ElementId,
    trigger: Option<TriggerBuilder>,
    menu: Option<MenuBuilder>,
    popup: Option<Entity<ContextMenuPopup>>,
    popup_position: Point<Pixels>,
    on_action: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

impl ContextMenuState {
    pub fn new(context_menu: ContextMenu) -> Self {
        Self {
            id: context_menu.id,
            trigger: context_menu.trigger,
            menu: context_menu.menu,
            popup: None,
            popup_position: point(px(0.), px(0.)),
            on_action: context_menu.on_action,
        }
    }

    /// Check if the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.popup.is_some()
    }

    /// Close the context menu if open.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if self.popup.is_some() {
            self.popup = None;
            cx.notify();
        }
    }

    fn show_menu(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        // Close any existing menu first
        self.popup = None;

        let entries = if let Some(menu_builder) = &self.menu {
            menu_builder(window, cx)
        } else {
            return;
        };

        if entries.is_empty() {
            return;
        }

        self.popup_position = position;
        let on_action = self.on_action.clone();
        let popup = ContextMenuPopup::build(entries, on_action, window, cx);

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

    fn on_right_click(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button == MouseButton::Right {
            self.show_menu(event.position, window, cx);
        }
    }
}

impl Render for ContextMenuState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger_element = if let Some(trigger) = &self.trigger {
            Some(trigger(window, cx))
        } else {
            None
        };

        let popup_position = self.popup_position;

        div()
            .id(self.id.clone())
            .relative()
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_right_click))
            .when_some(trigger_element, |this, trigger| this.child(trigger))
            .when_some(self.popup.clone(), |this, popup| {
                this.child(
                    deferred(
                        anchored()
                            .position(popup_position)
                            .child(div().occlude().child(popup)),
                    )
                    .with_priority(1),
                )
            })
    }
}
