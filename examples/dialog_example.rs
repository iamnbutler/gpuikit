//! Dialog Example - Demonstrates the Dialog component.
//!
//! This example shows how to use the Dialog component for modal interactions.

#![allow(missing_docs)]

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle, Focusable,
    ParentElement, Render, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_platform;
use gpuikit::elements::button::button;
use gpuikit::elements::dialog::{dialog, DialogClosed, DialogOpened, DialogState};
use gpuikit::layout::h_stack;
use gpuikit::theme::{ActiveTheme, Themeable};

struct DialogExample {
    focus_handle: FocusHandle,
    simple_dialog: Entity<DialogState>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl DialogExample {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create a simple dialog with just title and description
        let simple_dialog = cx.new(|_cx| {
            DialogState::new(
                dialog("simple-dialog")
                    .title("Welcome!")
                    .description("This is a simple dialog with a title and description. Press Escape or click the X button to close it. You can also click the backdrop to dismiss.")
            )
        });

        let mut subscriptions = Vec::new();

        // Track dialog open/close events
        subscriptions.push(cx.subscribe(&simple_dialog, |_this, _, _event: &DialogOpened, _cx| {
            println!("Simple dialog opened");
        }));
        subscriptions.push(cx.subscribe(&simple_dialog, |_this, _, _event: &DialogClosed, _cx| {
            println!("Simple dialog closed");
        }));

        Self {
            focus_handle: cx.focus_handle(),
            simple_dialog,
            _subscriptions: subscriptions,
        }
    }
}

impl Focusable for DialogExample {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DialogExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("dialog-example")
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .bg(theme.bg())
            .text_color(theme.fg())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_6()
            // Title
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Dialog Component Demo"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("Click the button below to open a dialog."),
            )
            // Button
            .child(
                h_stack().gap_4().child(
                    button("open-simple", "Open Dialog").on_click(cx.listener(
                        |this, _, window, cx| {
                            this.simple_dialog.update(cx, |dialog, cx| {
                                dialog.open(window, cx);
                            });
                        },
                    )),
                ),
            )
            // Dialog - rendered on top using deferred()
            .child(self.simple_dialog.clone())
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| DialogExample::new(window, cx));
                    let focus_handle = view.read(cx).focus_handle.clone();
                    window.focus(&focus_handle, cx);
                    view
                },
            )
            .unwrap();

            cx.activate(true);
        });
}
