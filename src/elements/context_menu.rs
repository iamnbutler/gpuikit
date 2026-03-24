//! Context Menu
//!
//! A right-click triggered menu for displaying contextual actions.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::context_menu::{context_menu, menu_item, menu_separator};
//!
//! let context_menu_state = cx.new(|_cx| {
//!     ContextMenuState::new(
//!         context_menu("file-actions")
//!             .trigger(|_window, _cx| div().p_4().child("Right-click me").into_any_element())
//!             .menu(|_window, _cx| {
//!                 vec![
//!                     menu_item("copy", "Copy").kbd("Cmd+C").into(),
//!                     menu_item("paste", "Paste").kbd("Cmd+V").into(),
//!                     menu_separator().into(),
//!                     menu_item("delete", "Delete").destructive().into(),
//!                 ]
//!             })
//!     )
//! });
//! ```

use crate::elements::kbd::{kbd, KbdSize};
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    anchored, deferred, div, point, prelude::*, px, App, AnyElement, Context, Corner,
    DismissEvent, ElementId, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    MouseButton, ParentElement, Point, Render, SharedString, Styled, Svg, Window,
};
use std::rc::Rc;

/// A menu entry - either an item or a separator.
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        MenuEntry::Item(item)
    }
}

impl From<MenuSeparator> for MenuEntry {
    fn from(_: MenuSeparator) -> Self {
        MenuEntry::Separator
    }
}

/// Icon factory type for menu items.
pub type IconFactory = Rc<dyn Fn() -> Svg>;

/// A menu item with label, optional icon, optional keyboard shortcut.
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconFactory>,
    kbd: Option<SharedString>,
    disabled: bool,
    destructive: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

/// Creates a new menu item.
pub fn menu_item(id: impl Into<ElementId>, label: impl Into<SharedString>) -> MenuItem {
    MenuItem::new(id, label)
}

/// Creates a menu separator.
pub fn menu_separator() -> MenuSeparator {
    MenuSeparator
}

/// A separator entry in the menu.
pub struct MenuSeparator;

impl MenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            kbd: None,
            disabled: false,
            destructive: false,
            on_click: None,
        }
    }

    /// Set an icon for this menu item using a factory function.
    pub fn icon(mut self, icon_factory: impl Fn() -> Svg + 'static) -> Self {
        self.icon = Some(Rc::new(icon_factory));
        self
    }

    /// Set a keyboard shortcut display for this menu item.
    pub fn kbd(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.kbd = Some(shortcut.into());
        self
    }

    /// Mark this item as disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Mark this item as destructive (will display in danger color).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Set a click handler for this menu item.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// The popup menu that displays context menu items.
pub struct ContextMenuPopup {
    entries: Vec<MenuEntry>,
    focus_handle: FocusHandle,
    focused_index: Option<usize>,
}

impl EventEmitter<DismissEvent> for ContextMenuPopup {}

impl Focusable for ContextMenuPopup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ContextMenuPopup {
    pub fn build(entries: Vec<MenuEntry>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                entries,
                focus_handle,
                focused_index: None,
            }
        })
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn select_item(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(MenuEntry::Item(item)) = self.entries.get(index) {
            if !item.disabled {
                if let Some(on_click) = &item.on_click {
                    let on_click = on_click.clone();
                    on_click(window, cx);
                }
                cx.emit(DismissEvent);
            }
        }
    }

    fn focusable_item_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e, MenuEntry::Item(item) if !item.disabled))
            .count()
    }

    fn index_to_focusable(&self, index: usize) -> Option<usize> {
        let mut focusable_count = 0;
        for (i, entry) in self.entries.iter().enumerate() {
            if let MenuEntry::Item(item) = entry {
                if !item.disabled {
                    if focusable_count == index {
                        return Some(i);
                    }
                    focusable_count += 1;
                }
            }
        }
        None
    }

    fn focusable_to_index(&self, entry_index: usize) -> Option<usize> {
        let mut focusable_count = 0;
        for (i, entry) in self.entries.iter().enumerate() {
            if let MenuEntry::Item(item) = entry {
                if !item.disabled {
                    if i == entry_index {
                        return Some(focusable_count);
                    }
                    focusable_count += 1;
                }
            }
        }
        None
    }

    fn move_focus(&mut self, delta: i32, cx: &mut Context<Self>) {
        let count = self.focusable_item_count();
        if count == 0 {
            return;
        }

        let current = self
            .focused_index
            .and_then(|i| self.focusable_to_index(i))
            .unwrap_or(if delta > 0 { count - 1 } else { 0 });

        let new_focusable = if delta > 0 {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };

        self.focused_index = self.index_to_focusable(new_focusable);
        cx.notify();
    }
}

impl Render for ContextMenuPopup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();

        div()
            .id("context-menu-popup")
            .track_focus(&focus_handle)
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => this.dismiss(window, cx),
                    "up" => this.move_focus(-1, cx),
                    "down" => this.move_focus(1, cx),
                    "enter" | " " => {
                        if let Some(index) = this.focused_index {
                            this.select_item(index, window, cx);
                        }
                    }
                    _ => {}
                }
            }))
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
            .children(self.entries.iter().enumerate().map(|(index, entry)| {
                match entry {
                    MenuEntry::Separator => div()
                        .my_1()
                        .h(px(1.))
                        .bg(theme.border_subtle())
                        .into_any_element(),
                    MenuEntry::Item(item) => {
                        let is_focused = self.focused_index == Some(index);
                        let theme = cx.theme();
                        let id = item.id.clone();
                        let disabled = item.disabled;
                        let destructive = item.destructive;

                        let text_color = if disabled {
                            theme.fg_disabled()
                        } else if destructive {
                            theme.danger()
                        } else {
                            theme.fg()
                        };

                        let mut row = div()
                            .id(id)
                            .px_3()
                            .py_1()
                            .mx_1()
                            .rounded_sm()
                            .text_xs()
                            .flex()
                            .items_center()
                            .gap_3()
                            .text_color(text_color);

                        if disabled {
                            row = row.cursor_not_allowed();
                        } else {
                            row = row
                                .cursor_pointer()
                                .when(is_focused, |this| this.bg(theme.surface_secondary()))
                                .hover(|style| style.bg(theme.surface_secondary()))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_item(index, window, cx);
                                }));
                        }

                        // Icon (if any)
                        let icon_element = item.icon.as_ref().map(|factory| {
                            factory()
                                .size(px(14.))
                                .text_color(if disabled {
                                    theme.fg_disabled()
                                } else {
                                    theme.fg_muted()
                                })
                                .flex_shrink_0()
                        });

                        // Label
                        let label_element = div().flex_1().child(item.label.clone());

                        // Keyboard shortcut (if any)
                        let kbd_element =
                            item.kbd.clone().map(|shortcut| kbd(shortcut).size(KbdSize::Small));

                        row = row.when_some(icon_element, |this, icon| this.child(icon));
                        row = row.child(label_element);
                        row = row.when_some(kbd_element, |this, kbd| this.child(kbd));

                        row.into_any_element()
                    }
                }
            }))
    }
}

type MenuBuilder = Rc<dyn Fn(&mut Window, &mut App) -> Vec<MenuEntry>>;
type TriggerRender = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Builder for creating a context menu component.
pub struct ContextMenu {
    id: ElementId,
    trigger_render: Option<TriggerRender>,
    menu_builder: Option<MenuBuilder>,
}

/// Creates a new context menu builder.
pub fn context_menu(id: impl Into<ElementId>) -> ContextMenu {
    ContextMenu::new(id)
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_render: None,
            menu_builder: None,
        }
    }

    /// Set the trigger element via a render callback that responds to right-click.
    pub fn trigger(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger_render = Some(Rc::new(render));
        self
    }

    /// Set the menu builder function that creates menu entries.
    pub fn menu(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> Vec<MenuEntry> + 'static,
    ) -> Self {
        self.menu_builder = Some(Rc::new(builder));
        self
    }
}

/// Stateful context menu component that manages the popup.
pub struct ContextMenuState {
    id: ElementId,
    trigger_render: Option<TriggerRender>,
    menu_builder: Option<MenuBuilder>,
    popup: Option<Entity<ContextMenuPopup>>,
    cursor_position: Point<gpui::Pixels>,
}

impl ContextMenuState {
    pub fn new(context_menu: ContextMenu) -> Self {
        Self {
            id: context_menu.id,
            trigger_render: context_menu.trigger_render,
            menu_builder: context_menu.menu_builder,
            popup: None,
            cursor_position: point(px(0.), px(0.)),
        }
    }

    /// Check if the context menu is currently open.
    pub fn is_open(&self) -> bool {
        self.popup.is_some()
    }

    /// Close the context menu.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.popup = None;
        cx.notify();
    }

    fn open_menu(
        &mut self,
        position: Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(builder) = &self.menu_builder {
            let entries = builder(window, cx);
            self.cursor_position = position;

            let popup = ContextMenuPopup::build(entries, window, cx);

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
    }

}

impl Render for ContextMenuState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_pos = self.cursor_position;

        let trigger_element = if let Some(ref render) = self.trigger_render {
            Some(render(window, cx))
        } else {
            None
        };

        div()
            .id(self.id.clone())
            .relative()
            .child(
                div()
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                            window.prevent_default();
                            this.open_menu(event.position, window, cx);
                            cx.stop_propagation();
                        }),
                    )
                    .when_some(trigger_element, |el, trigger| el.child(trigger)),
            )
            .when_some(self.popup.clone(), move |this, popup| {
                this.child(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .position(cursor_pos)
                            .child(div().occlude().child(popup)),
                    )
                    .with_priority(1),
                )
            })
    }
}
