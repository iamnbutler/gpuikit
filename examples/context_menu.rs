#![allow(missing_docs)]
//! Context menu demo.
//!
//! Right-click a row to act on it. Shows the two ways an item does its work:
//!
//! - `on_click`, a closure — used here for the per-row pin/delete actions,
//!   which reach back into the view through its entity handle.
//! - `action`, a gpui action — used for "Duplicate". Its shortcut is read from
//!   the keymap rather than written into the menu, and it dispatches to
//!   whatever had focus before the menu opened, so the same handler serves the
//!   menu and the keyboard.
//!
//! Run with: cargo run --example context_menu

use gpui::{
    actions, div, prelude::*, px, size, App, Application, Bounds, Context, FocusHandle, Focusable,
    IntoElement, KeyBinding, ParentElement, Render, SharedString, Styled, Window, WindowBounds,
    WindowOptions,
};
use gpuikit::elements::context_menu::{context_menu, menu_item};
use gpuikit::layout::v_stack;
use gpuikit::theme::{ActiveTheme, Themeable};
use gpuikit::DefaultIcons;

actions!(context_menu_example, [Duplicate]);

struct Row {
    name: SharedString,
    pinned: bool,
}

struct Demo {
    focus_handle: FocusHandle,
    rows: Vec<Row>,
    selected: usize,
    status: SharedString,
}

impl Demo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            rows: vec![
                Row {
                    name: "Untitled draft".into(),
                    pinned: false,
                },
                Row {
                    name: "Quarterly plan".into(),
                    pinned: true,
                },
                Row {
                    name: "Release notes".into(),
                    pinned: false,
                },
            ],
            selected: 0,
            status: "Right-click a row.".into(),
        }
    }

    /// Handles the action whether it arrives from the menu or from ⌘D.
    ///
    /// The menu dispatches to the focus that was in place before it opened, so
    /// this fires either way without the menu having to know it exists.
    fn duplicate(&mut self, _: &Duplicate, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.rows.get(self.selected) else {
            return;
        };
        let copy = format!("{} copy", row.name);
        self.status = format!("Duplicated “{}”", row.name).into();
        self.rows.insert(
            self.selected + 1,
            Row {
                name: copy.into(),
                pinned: false,
            },
        );
        cx.notify();
    }

    fn render_row(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let row = &self.rows[index];
        let selected = self.selected == index;
        let name = row.name.clone();
        let pinned = row.pinned;
        let this = cx.entity();
        let last_row = self.rows.len() == 1;

        // The trigger is an ordinary element built from view state; the menu
        // wraps what the view already renders rather than replacing it.
        let trigger = div()
            .id(("row", index))
            .flex()
            .items_center()
            .gap_2()
            .w(px(320.))
            .px_3()
            .py_2()
            .rounded_md()
            .text_sm()
            .when(selected, |this| this.bg(theme.surface_secondary()))
            .hover(|style| style.bg(theme.surface_secondary()))
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.selected = index;
                cx.notify();
            }))
            .when(pinned, |this| {
                this.child(
                    DefaultIcons::star()
                        .size(px(12.))
                        .text_color(theme.fg_muted()),
                )
            })
            .child(name.clone());

        context_menu(("row-menu", index), trigger).menu(move |menu, _window, _cx| {
            let name = name.clone();
            menu.header(name.clone())
                .item(
                    menu_item("Duplicate")
                        .icon(DefaultIcons::copy)
                        // No .kbd() — the shortcut comes from the keymap.
                        .action(Box::new(Duplicate)),
                )
                .item(menu_item("Pinned").toggled(pinned).on_click({
                    let this = this.clone();
                    move |_window, cx| {
                        this.update(cx, |demo: &mut Demo, cx| {
                            demo.rows[index].pinned = !demo.rows[index].pinned;
                            demo.status = if demo.rows[index].pinned {
                                format!("Pinned “{}”", demo.rows[index].name).into()
                            } else {
                                format!("Unpinned “{}”", demo.rows[index].name).into()
                            };
                            cx.notify();
                        });
                    }
                }))
                .separator()
                .item(
                    menu_item("Delete")
                        .icon(DefaultIcons::trash)
                        .destructive()
                        // The last row cannot go: a disabled item explains why
                        // the command exists but is unavailable, where hiding
                        // it would just look like a missing feature.
                        .disabled(last_row)
                        .on_click({
                            let this = this.clone();
                            move |_window, cx| {
                                this.update(cx, |demo: &mut Demo, cx| {
                                    let removed = demo.rows.remove(index);
                                    demo.selected = demo.selected.min(demo.rows.len() - 1);
                                    demo.status = format!("Deleted “{}”", removed.name).into();
                                    cx.notify();
                                });
                            }
                        }),
                )
        })
    }
}

impl Focusable for Demo {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = (0..self.rows.len())
            .map(|index| self.render_row(index, cx).into_any_element())
            .collect::<Vec<_>>();
        let theme = cx.theme();

        div()
            .track_focus(&self.focus_handle)
            // Scopes ⌘D to this view, and is what lets the menu find the
            // binding to display next to "Duplicate".
            .key_context("Demo")
            .on_action(cx.listener(Self::duplicate))
            .size_full()
            .bg(theme.bg())
            .text_color(theme.fg())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_6()
            .child(
                v_stack()
                    .gap_1()
                    .items_center()
                    .child(div().text_lg().child("Context Menu"))
                    .child(div().text_sm().text_color(theme.fg_muted()).child(
                        "Right-click a row. Arrow keys move, Enter chooses, Escape closes.",
                    )),
            )
            .child(v_stack().gap_1().children(rows))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.fg_muted())
                    .child(self.status.clone()),
            )
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);
            cx.bind_keys([KeyBinding::new("cmd-d", Duplicate, Some("Demo"))]);

            let bounds = Bounds::centered(None, size(px(560.), px(420.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| cx.new(Demo::new),
            )
            .unwrap();
        });
}
