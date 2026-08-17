//! Context Menu
//!
//! A menu of actions, opened by right-clicking the element it is attached to.
//!
//! The menu attaches to an element you have already built, so it composes with
//! whatever a view renders — a list row, a card, a piece of text — without
//! restructuring that view:
//!
//! ```ignore
//! use gpuikit::elements::context_menu::{context_menu, menu_item};
//!
//! context_menu("row", self.render_row(ix, cx)).menu(move |menu, _window, _cx| {
//!     menu.item(menu_item("Copy").kbd("⌘C").on_click(|_window, _cx| { /* … */ }))
//!         .item(menu_item("Rename").action(Box::new(Rename)))
//!         .separator()
//!         .item(menu_item("Delete").destructive().disabled(!can_delete))
//! })
//! ```
//!
//! Entries are built once, when the menu opens, so they see the state at the
//! moment of the click. Give each menu a stable, unique id — the open/closed
//! state is keyed on it, so rows in a list need the row index in their id.
//!
//! An item can carry a closure ([`MenuItem::on_click`]), a gpui action
//! ([`MenuItem::action`]), or both. An action is the better default where one
//! exists: it dispatches to whatever had focus before the menu opened, and its
//! keyboard shortcut is read from the keymap rather than hardcoded.

use std::rc::Rc;

use gpui::{
    anchored, deferred, div, prelude::*, px, Action, AnyElement, App, Context, ElementId, Entity,
    FocusHandle, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, ParentElement, Pixels,
    Point, ScrollHandle, SharedString, Styled, Svg, Window,
};

use crate::element_id::scoped;
use crate::elements::kbd::kbd;
use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::control_sized::ControlSized;

/// The id of the open popup of the menu attached to `menu_id`.
///
/// The popup is a plain `div()` inside a `RenderOnce`, and `deferred()` does
/// not give it a scope of its own — `Window::defer_draw` keeps whatever id path
/// the popup was built under. Two menus open at once (or one menu and one
/// keyboard-driven one) would otherwise be the same id. Derived from the id the
/// caller gave the menu, which the open/closed state is keyed on already.
fn menu_element_id(menu_id: &ElementId) -> ElementId {
    scoped(menu_id, "popup")
}

/// Builds the leading icon of a menu item.
pub type IconFactory = Rc<dyn Fn() -> Svg>;

type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type MenuBuilder = Rc<dyn Fn(MenuItems, &mut Window, &mut App) -> MenuItems>;

/// A single action in a context menu.
pub struct MenuItem {
    label: SharedString,
    icon: Option<IconFactory>,
    kbd: Option<SharedString>,
    action: Option<Box<dyn Action>>,
    on_click: Option<ClickHandler>,
    disabled: bool,
    destructive: bool,
    toggled: Option<bool>,
}

/// Creates a menu item with the given label.
pub fn menu_item(label: impl Into<SharedString>) -> MenuItem {
    MenuItem::new(label)
}

impl MenuItem {
    /// Creates a menu item with the given label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            kbd: None,
            action: None,
            on_click: None,
            disabled: false,
            destructive: false,
            toggled: None,
        }
    }

    /// Sets a leading icon.
    pub fn icon(mut self, icon: impl Fn() -> Svg + 'static) -> Self {
        self.icon = Some(Rc::new(icon));
        self
    }

    /// Sets the keyboard shortcut shown after the label.
    ///
    /// Only needed for items with no [`action`](Self::action) — an item that
    /// has one takes its shortcut from the keymap.
    pub fn kbd(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.kbd = Some(shortcut.into());
        self
    }

    /// Dispatches the given action when this item is chosen.
    ///
    /// The action goes to whatever was focused before the menu opened, so it
    /// lands on the view the menu was opened over rather than on the menu. The
    /// item's keyboard shortcut is taken from the action's highest-precedence
    /// binding unless [`kbd`](Self::kbd) overrides it.
    pub fn action(mut self, action: Box<dyn Action>) -> Self {
        self.action = Some(action);
        self
    }

    /// Runs the given callback when this item is chosen.
    ///
    /// Runs after the menu has closed and focus has been restored, so a handler
    /// is free to open a dialog or move focus itself.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Greys the item out and stops it from being chosen or focused.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Renders the item in the theme's danger color.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Renders the item with a checkmark when `toggled` is true.
    ///
    /// Items that show a toggle reserve room for the checkmark either way, so
    /// a group of them stays aligned as the state changes.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }
}

/// One entry in a context menu.
///
/// Built through [`MenuItems`] rather than directly.
#[non_exhaustive]
pub enum MenuEntry {
    /// An action.
    Item(MenuItem),
    /// A horizontal rule between groups of items.
    Separator,
    /// A label introducing the group of items below it.
    Header(SharedString),
}

/// The entries of a context menu.
///
/// Handed to the [`ContextMenu::menu`] callback to be filled in.
#[derive(Default)]
pub struct MenuItems {
    entries: Vec<MenuEntry>,
}

impl MenuItems {
    /// Creates an empty set of entries.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self
    }

    /// Appends several items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.entries.extend(items.into_iter().map(MenuEntry::Item));
        self
    }

    /// Appends a separator.
    ///
    /// Separators that would land at the start of the menu or next to another
    /// separator are dropped, so a menu assembled from optional groups does not
    /// need to track which of them ended up present.
    pub fn separator(mut self) -> Self {
        if matches!(
            self.entries.last(),
            None | Some(MenuEntry::Separator) | Some(MenuEntry::Header(_))
        ) {
            return self;
        }
        self.entries.push(MenuEntry::Separator);
        self
    }

    /// Appends a section header.
    pub fn header(mut self, label: impl Into<SharedString>) -> Self {
        self.entries.push(MenuEntry::Header(label.into()));
        self
    }

    /// Applies `f` only when `condition` holds.
    pub fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            f(self)
        } else {
            self
        }
    }

    /// Whether no entries have been added.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drops trailing separators and headers, which are what a conditionally
    /// built menu leaves behind when its last group turns out to be empty. A
    /// menu that reduces to nothing this way does not open at all.
    fn finish(mut self) -> Vec<MenuEntry> {
        while matches!(
            self.entries.last(),
            Some(MenuEntry::Separator) | Some(MenuEntry::Header(_))
        ) {
            self.entries.pop();
        }
        self.entries
    }
}

/// Indices of the entries that can be focused: items that are not disabled.
/// Separators and headers are skipped over by keyboard navigation.
fn selectable_indices(entries: &[MenuEntry]) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(ix, entry)| match entry {
            MenuEntry::Item(item) if !item.disabled => Some(ix),
            _ => None,
        })
        .collect()
}

/// The entry `delta` steps from `current`, wrapping at both ends.
///
/// With nothing focused yet, arrowing down enters the menu at the top and
/// arrowing up enters it at the bottom.
fn next_focus(selectable: &[usize], current: Option<usize>, delta: isize) -> Option<usize> {
    if selectable.is_empty() {
        return None;
    }
    match current.and_then(|ix| selectable.iter().position(|&s| s == ix)) {
        Some(pos) => {
            let moved = (pos as isize + delta).rem_euclid(selectable.len() as isize);
            selectable.get(moved as usize).copied()
        }
        None if delta >= 0 => selectable.first().copied(),
        None => selectable.last().copied(),
    }
}

/// An open menu: where it sits, what is in it, and what is focused.
struct OpenMenu {
    position: Point<Pixels>,
    entries: Vec<MenuEntry>,
    focused: Option<usize>,
    restore_focus: Option<FocusHandle>,
    scroll: ScrollHandle,
}

/// The open/closed state of one context menu, held across frames.
struct MenuState {
    focus_handle: FocusHandle,
    open: Option<OpenMenu>,
}

impl MenuState {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            open: None,
        }
    }

    fn open(
        &mut self,
        position: Point<Pixels>,
        entries: Vec<MenuEntry>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if entries.is_empty() {
            return;
        }

        let restore_focus = window.focused(cx);
        self.open = Some(OpenMenu {
            position,
            entries,
            focused: None,
            restore_focus,
            scroll: ScrollHandle::new(),
        });
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(open) = self.open.take() else {
            return;
        };
        if let Some(handle) = open.restore_focus {
            window.focus(&handle, cx);
        }
        cx.notify();
    }

    /// Moves focus by `delta` selectable entries, wrapping at both ends.
    fn move_focus(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(open) = self.open.as_mut() else {
            return;
        };
        let Some(next) = next_focus(&selectable_indices(&open.entries), open.focused, delta) else {
            return;
        };

        open.focused = Some(next);
        open.scroll.scroll_to_item(next);
        cx.notify();
    }

    /// Focuses the first (`delta >= 0`) or last selectable entry.
    fn focus_edge(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some(open) = self.open.as_mut() {
            open.focused = None;
        }
        self.move_focus(delta, cx);
    }

    /// Follows the pointer, so the mouse and the keyboard never disagree about
    /// which entry is about to be chosen.
    fn focus_from_hover(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(open) = self.open.as_mut() else {
            return;
        };
        if open.focused == Some(index) {
            return;
        }
        open.focused = Some(index);
        cx.notify();
    }

    fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(open) = self.open.as_ref() else {
            return;
        };
        let Some(MenuEntry::Item(item)) = open.entries.get(index) else {
            return;
        };
        if item.disabled {
            return;
        }

        let on_click = item.on_click.clone();
        let action = item.action.as_ref().map(|action| action.boxed_clone());
        let restore_focus = open.restore_focus.clone();

        // Close first: the handler may want to move focus or open something of
        // its own, and the action has to reach the view the menu covered rather
        // than the menu itself.
        self.close(window, cx);

        if let Some(on_click) = on_click {
            on_click(window, cx);
        }
        if let Some(action) = action {
            match restore_focus {
                Some(handle) => handle.dispatch_action(action.as_ref(), window, cx),
                None => window.dispatch_action(action, cx),
            }
        }
    }
}

/// What one entry looks like this frame, lifted out of the state entity so the
/// borrow ends before the elements are built.
enum Row {
    Item {
        index: usize,
        label: SharedString,
        icon: Option<IconFactory>,
        kbd: Option<SharedString>,
        disabled: bool,
        destructive: bool,
        toggled: Option<bool>,
        focused: bool,
    },
    Separator,
    Header(SharedString),
}

/// The shortcut to show for an item: its explicit one, else its action's
/// highest-precedence binding. An action with no binding shows nothing.
fn shortcut_for(item: &MenuItem, window: &Window) -> Option<SharedString> {
    if let Some(shortcut) = item.kbd.clone() {
        return Some(shortcut);
    }
    let action = item.action.as_ref()?;
    let binding = window.highest_precedence_binding_for_action(action.as_ref())?;
    let text = binding
        .keystrokes()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        None
    } else {
        Some(text.into())
    }
}

fn rows_for(open: &OpenMenu, window: &Window) -> Vec<Row> {
    open.entries
        .iter()
        .enumerate()
        .map(|(index, entry)| match entry {
            MenuEntry::Separator => Row::Separator,
            MenuEntry::Header(label) => Row::Header(label.clone()),
            MenuEntry::Item(item) => Row::Item {
                index,
                label: item.label.clone(),
                icon: item.icon.clone(),
                kbd: shortcut_for(item, window),
                disabled: item.disabled,
                destructive: item.destructive,
                toggled: item.toggled,
                focused: open.focused == Some(index),
            },
        })
        .collect()
}

/// A context menu attached to a trigger element.
///
/// Created with [`context_menu`].
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    trigger: AnyElement,
    builder: Option<MenuBuilder>,
    min_width: Pixels,
    max_height: Pixels,
}

/// Attaches a context menu to `trigger`.
///
/// The id must be stable across frames and unique among its siblings — the
/// menu's open state is keyed on it.
pub fn context_menu(id: impl Into<ElementId>, trigger: impl IntoElement) -> ContextMenu {
    ContextMenu {
        id: id.into(),
        trigger: trigger.into_any_element(),
        builder: None,
        min_width: px(180.),
        max_height: px(420.),
    }
}

impl ContextMenu {
    /// Sets the callback that fills in the menu's entries.
    ///
    /// Called each time the menu opens, so the entries reflect the state at the
    /// moment of the click. A menu that comes back empty does not open.
    pub fn menu(
        mut self,
        builder: impl Fn(MenuItems, &mut Window, &mut App) -> MenuItems + 'static,
    ) -> Self {
        self.builder = Some(Rc::new(builder));
        self
    }

    /// Sets the menu's minimum width. Defaults to 180px.
    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = width;
        self
    }

    /// Sets the height past which the menu scrolls. Defaults to 420px.
    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| MenuState::new(cx));
        let popup_id = menu_element_id(&self.id);

        let ContextMenu {
            trigger,
            builder,
            min_width,
            max_height,
            ..
        } = self;

        // Everything the popup needs, taken by value so the borrow of the state
        // entity ends here rather than running through element construction.
        let open = {
            let menu = state.read(cx);
            menu.open.as_ref().map(|open| {
                (
                    open.position,
                    rows_for(open, window),
                    open.scroll.clone(),
                    menu.focus_handle.clone(),
                )
            })
        };

        let trigger = div()
            .when_some(builder, |el, builder| {
                let state = state.clone();
                el.on_mouse_down(
                    MouseButton::Right,
                    move |event: &MouseDownEvent, window, cx| {
                        window.prevent_default();
                        let entries = builder(MenuItems::new(), window, cx).finish();
                        let position = event.position;
                        state.update(cx, |menu, cx| menu.open(position, entries, window, cx));
                        cx.stop_propagation();
                    },
                )
            })
            .child(trigger);

        div()
            .child(trigger)
            .when_some(open, |el, (position, rows, scroll, focus_handle)| {
                el.child(
                    deferred(
                        anchored()
                            .position(position)
                            // Keep the whole menu on screen near a window edge.
                            .snap_to_window_with_margin(px(8.))
                            .child(div().occlude().child(menu_popup(
                                popup_id,
                                rows,
                                scroll,
                                focus_handle,
                                state,
                                min_width,
                                max_height,
                                cx,
                            ))),
                    )
                    .with_priority(1),
                )
            })
    }
}

#[allow(clippy::too_many_arguments)]
fn menu_popup(
    popup_id: ElementId,
    rows: Vec<Row>,
    scroll: ScrollHandle,
    focus_handle: FocusHandle,
    state: Entity<MenuState>,
    min_width: Pixels,
    max_height: Pixels,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .id(popup_id)
        .track_focus(&focus_handle)
        .on_mouse_down_out({
            let state = state.clone();
            move |_, window, cx| {
                state.update(cx, |menu, cx| menu.close(window, cx));
            }
        })
        .on_key_down({
            let state = state.clone();
            move |event: &KeyDownEvent, window, cx| {
                let handled = match event.keystroke.key.as_str() {
                    "escape" => {
                        state.update(cx, |menu, cx| menu.close(window, cx));
                        true
                    }
                    "up" => {
                        state.update(cx, |menu, cx| menu.move_focus(-1, cx));
                        true
                    }
                    "down" => {
                        state.update(cx, |menu, cx| menu.move_focus(1, cx));
                        true
                    }
                    "home" => {
                        state.update(cx, |menu, cx| menu.focus_edge(1, cx));
                        true
                    }
                    "end" => {
                        state.update(cx, |menu, cx| menu.focus_edge(-1, cx));
                        true
                    }
                    "enter" | "space" => state.update(cx, |menu, cx| {
                        let focused = menu.open.as_ref().and_then(|open| open.focused);
                        match focused {
                            Some(index) => {
                                menu.activate(index, window, cx);
                                true
                            }
                            None => false,
                        }
                    }),
                    _ => false,
                };
                if handled {
                    cx.stop_propagation();
                }
            }
        })
        .min_w(min_width)
        .max_h(max_height)
        .overflow_y_scroll()
        .track_scroll(&scroll)
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
            rows.into_iter()
                .map(|row| menu_row(row, state.clone(), cx).into_any_element()),
        )
}

fn menu_row(row: Row, state: Entity<MenuState>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();

    match row {
        Row::Separator => div()
            .my_1()
            .h(px(1.))
            .bg(theme.border_subtle())
            .into_any_element(),
        Row::Header(label) => div()
            .px_3()
            .pt_2()
            .pb_1()
            .text_xs()
            .text_color(theme.fg_muted())
            .child(label)
            .into_any_element(),
        Row::Item {
            index,
            label,
            icon,
            kbd: shortcut,
            disabled,
            destructive,
            toggled,
            focused,
        } => {
            let text_color = if disabled {
                theme.fg_disabled()
            } else if destructive {
                theme.danger()
            } else {
                theme.fg()
            };
            let icon_color = if disabled {
                theme.fg_disabled()
            } else if destructive {
                theme.danger()
            } else {
                theme.fg_muted()
            };

            let mut row = div()
                .id(("gpuikit-context-menu-item", index))
                .px_3()
                .py_1()
                .mx_1()
                .rounded_sm()
                .text_xs()
                .flex()
                .items_center()
                .gap_2()
                .text_color(text_color);

            if disabled {
                row = row.cursor_not_allowed();
            } else {
                row = row
                    .cursor_pointer()
                    .when(focused, |this| this.bg(theme.surface_secondary()))
                    .on_mouse_move({
                        let state = state.clone();
                        move |_, _window, cx| {
                            state.update(cx, |menu, cx| menu.focus_from_hover(index, cx));
                        }
                    })
                    .on_click({
                        let state = state.clone();
                        move |_, window, cx| {
                            state.update(cx, |menu, cx| menu.activate(index, window, cx));
                        }
                    });
            }

            // Items that can toggle keep the checkmark's width whether or not
            // they are on, so a group of them stays aligned.
            if let Some(toggled) = toggled {
                row = row.child(div().w(px(14.)).flex_shrink_0().when(toggled, |this| {
                    this.child(Icons::check().size(px(14.)).text_color(icon_color))
                }));
            }

            let row = row
                .when_some(icon, |this, icon| {
                    this.child(icon().size(px(14.)).text_color(icon_color).flex_shrink_0())
                })
                .child(div().flex_1().child(label))
                .when_some(shortcut, |this, shortcut| this.child(kbd(shortcut).small()));

            // Lets a test find where a row was actually laid out, so clicking
            // one does not mean hardcoding the menu's metrics.
            #[cfg(test)]
            let row = row.debug_selector(|| format!("gpuikit-context-menu-item-{index}"));

            row.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, Modifiers, MouseUpEvent, Render, TestAppContext, VisualTestContext};
    use std::cell::RefCell;

    fn labels(entries: &[MenuEntry]) -> Vec<&str> {
        entries
            .iter()
            .map(|entry| match entry {
                MenuEntry::Item(item) => item.label.as_ref(),
                MenuEntry::Separator => "---",
                MenuEntry::Header(label) => label.as_ref(),
            })
            .collect()
    }

    #[test]
    fn each_menu_pops_up_under_its_own_id() {
        let row = ElementId::named_usize("row", 3);
        let other_row = ElementId::named_usize("row", 4);

        assert_ne!(menu_element_id(&row), menu_element_id(&other_row));
        assert_eq!(menu_element_id(&row), menu_element_id(&row));
        assert_ne!(menu_element_id(&row), row);
    }

    #[test]
    fn separators_never_lead_or_double_up() {
        let entries = MenuItems::new()
            .separator()
            .item(menu_item("Copy"))
            .separator()
            .separator()
            .item(menu_item("Delete"))
            .finish();

        assert_eq!(labels(&entries), vec!["Copy", "---", "Delete"]);
    }

    #[test]
    fn trailing_separators_are_dropped() {
        // The shape a conditionally-built menu takes when its last group is empty.
        let entries = MenuItems::new()
            .item(menu_item("Copy"))
            .separator()
            .when(false, |menu| menu.item(menu_item("Delete")))
            .finish();

        assert_eq!(labels(&entries), vec!["Copy"]);
    }

    #[test]
    fn an_empty_trailing_section_leaves_nothing_behind() {
        let entries = MenuItems::new()
            .item(menu_item("Copy"))
            .separator()
            .header("Danger")
            .when(false, |menu| menu.item(menu_item("Delete")))
            .finish();

        assert_eq!(labels(&entries), vec!["Copy"]);
    }

    #[test]
    fn a_menu_that_reduces_to_nothing_is_empty() {
        let entries = MenuItems::new()
            .header("Danger")
            .when(false, |menu| menu.item(menu_item("Delete")))
            .finish();

        assert!(entries.is_empty());
    }

    #[test]
    fn a_separator_after_a_header_is_dropped() {
        let entries = MenuItems::new()
            .item(menu_item("Copy"))
            .separator()
            .header("Danger")
            .separator()
            .item(menu_item("Delete"))
            .finish();

        assert_eq!(labels(&entries), vec!["Copy", "---", "Danger", "Delete"]);
    }

    #[test]
    fn keyboard_navigation_skips_headers_separators_and_disabled_items() {
        let entries = MenuItems::new()
            .header("Group")
            .item(menu_item("Copy"))
            .item(menu_item("Paste").disabled(true))
            .separator()
            .item(menu_item("Delete"))
            .finish();

        // Header at 0, Copy at 1, disabled Paste at 2, separator at 3, Delete at 4.
        assert_eq!(selectable_indices(&entries), vec![1, 4]);
    }

    #[test]
    fn arrowing_into_a_fresh_menu_enters_from_the_end_it_moves_from() {
        let selectable = vec![1, 4];

        assert_eq!(next_focus(&selectable, None, 1), Some(1));
        assert_eq!(next_focus(&selectable, None, -1), Some(4));
    }

    #[test]
    fn focus_wraps_at_both_ends() {
        let selectable = vec![1, 4];

        assert_eq!(next_focus(&selectable, Some(1), 1), Some(4));
        assert_eq!(next_focus(&selectable, Some(4), 1), Some(1));
        assert_eq!(next_focus(&selectable, Some(1), -1), Some(4));
    }

    #[test]
    fn a_menu_with_nothing_selectable_never_focuses() {
        let entries = MenuItems::new()
            .header("Group")
            .item(menu_item("Copy").disabled(true))
            .finish();
        let selectable = selectable_indices(&entries);

        assert!(selectable.is_empty());
        assert_eq!(next_focus(&selectable, None, 1), None);
        assert_eq!(next_focus(&selectable, None, -1), None);
    }

    #[test]
    fn focus_survives_an_entry_that_is_no_longer_selectable() {
        // Focus is held as an entry index; a menu rebuilt with that entry
        // disabled must not strand the keyboard on it.
        let selectable = vec![0, 2];

        assert_eq!(next_focus(&selectable, Some(1), 1), Some(0));
    }

    /// Records which items were chosen, so a test can assert on what the menu
    /// actually did rather than on its internal state.
    #[derive(Clone, Default)]
    struct Chosen(Rc<RefCell<Vec<String>>>);

    impl Chosen {
        fn record(&self, label: &'static str) -> impl Fn(&mut Window, &mut App) + use<> {
            let chosen = self.0.clone();
            move |_window, _cx| chosen.borrow_mut().push(label.to_string())
        }

        fn get(&self) -> Vec<String> {
            self.0.borrow().clone()
        }
    }

    struct TestView {
        chosen: Chosen,
    }

    impl Render for TestView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let chosen = self.chosen.clone();

            context_menu("test-menu", div().w(px(400.)).h(px(300.))).menu(
                move |menu, _window, _cx| {
                    menu.item(menu_item("Copy").on_click(chosen.record("Copy")))
                        // Both skipped by the keyboard, for different reasons.
                        .item(
                            menu_item("Paste")
                                .disabled(true)
                                .on_click(chosen.record("Paste")),
                        )
                        .separator()
                        .item(menu_item("Delete").on_click(chosen.record("Delete")))
                },
            )
        }
    }

    fn open_menu(cx: &mut TestAppContext) -> (Chosen, &mut VisualTestContext) {
        // Only the theme, not the whole of `gpuikit::init`: the rest binds keys
        // (escape opens dialogs) that these tests would then be testing too.
        cx.update(crate::theme::init);

        let chosen = Chosen::default();
        let (_view, cx) = cx.add_window_view({
            let chosen = chosen.clone();
            move |_window, _cx| TestView { chosen }
        });

        right_click(cx, point(px(50.), px(50.)));
        (chosen, cx)
    }

    fn right_click(cx: &mut VisualTestContext, position: Point<Pixels>) {
        cx.simulate_event(MouseDownEvent {
            position,
            modifiers: Modifiers::default(),
            button: MouseButton::Right,
            click_count: 1,
            first_mouse: false,
        });
        cx.simulate_event(MouseUpEvent {
            position,
            modifiers: Modifiers::default(),
            button: MouseButton::Right,
            click_count: 1,
        });
    }

    #[gpui::test]
    fn right_click_opens_the_menu_and_enter_chooses(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        cx.simulate_keystrokes("down enter");

        assert_eq!(chosen.get(), vec!["Copy".to_string()]);
    }

    #[gpui::test]
    fn arrowing_past_a_disabled_item_lands_on_the_next_enabled_one(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        // Down twice: Copy, then past disabled Paste and the separator.
        cx.simulate_keystrokes("down down enter");

        assert_eq!(chosen.get(), vec!["Delete".to_string()]);
    }

    #[gpui::test]
    fn escape_closes_the_menu_without_choosing(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        cx.simulate_keystrokes("down escape");
        // Would choose Copy if the menu were still open and still focused.
        cx.simulate_keystrokes("down enter");

        assert!(chosen.get().is_empty(), "{:?}", chosen.get());
    }

    #[gpui::test]
    fn clicking_away_closes_the_menu(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        cx.simulate_click(point(px(900.), px(700.)), Modifiers::default());
        cx.simulate_keystrokes("down enter");

        assert!(chosen.get().is_empty(), "{:?}", chosen.get());
    }

    #[gpui::test]
    fn clicking_an_item_chooses_it(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        // Covers what the keyboard tests cannot: that the popup receives mouse
        // events at all, over whatever it was opened on top of.
        // 0 Copy, 1 Paste, 2 the separator, 3 Delete.
        let bounds = cx
            .debug_bounds("gpuikit-context-menu-item-3")
            .expect("the Delete row should have been laid out");
        cx.simulate_click(bounds.center(), Modifiers::default());

        assert_eq!(chosen.get(), vec!["Delete".to_string()]);
    }

    #[gpui::test]
    fn clicking_a_disabled_item_does_nothing(cx: &mut TestAppContext) {
        let (chosen, cx) = open_menu(cx);

        let bounds = cx
            .debug_bounds("gpuikit-context-menu-item-1")
            .expect("the Paste row should have been laid out");
        cx.simulate_click(bounds.center(), Modifiers::default());

        assert!(chosen.get().is_empty(), "{:?}", chosen.get());
        // Still open: a dead click inside the menu must not dismiss it.
        cx.simulate_keystrokes("down enter");
        assert_eq!(chosen.get(), vec!["Copy".to_string()]);
    }

    #[gpui::test]
    fn right_clicking_again_moves_the_open_menu(cx: &mut TestAppContext) {
        // The popup's outside-press handler and the trigger's press handler both
        // see this click. Capture runs before bubble, so it closes and reopens
        // rather than closing what it just opened.
        let (chosen, cx) = open_menu(cx);

        right_click(cx, point(px(120.), px(90.)));
        cx.simulate_keystrokes("down enter");

        assert_eq!(chosen.get(), vec!["Copy".to_string()]);
    }
}
