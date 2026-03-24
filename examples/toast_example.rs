//! Toast Example - Demonstrates the Toast notification component.
//!
//! This example shows how to use the Toast component for temporary notifications.

#![allow(missing_docs)]

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FocusHandle, Focusable,
    ParentElement, Render, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_platform;
use gpuikit::elements::button::button;
use gpuikit::elements::toast::{toast_container, ToastExt, ToastPosition};
use gpuikit::layout::{h_stack, v_stack};
use gpuikit::theme::{ActiveTheme, Themeable};
use std::time::Duration;

struct ToastExample {
    focus_handle: FocusHandle,
}

impl ToastExample {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for ToastExample {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToastExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("toast-example")
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
                    .child("Toast Component Demo"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .max_w(px(500.))
                    .text_center()
                    .child("Click the buttons below to show different toast notifications. Toasts will auto-dismiss after a few seconds."),
            )
            // Variant Buttons
            .child(
                v_stack()
                    .gap_4()
                    .items_center()
                    .child(
                        h_stack()
                            .gap_2()
                            .child(
                                button("default", "Default").on_click(|_, _, cx| {
                                    cx.toast("This is a default notification").show();
                                }),
                            )
                            .child(
                                button("info", "Info").on_click(|_, _, cx| {
                                    cx.toast("Here's some useful information")
                                        .title("Information")
                                        .info()
                                        .show();
                                }),
                            )
                            .child(
                                button("success", "Success").on_click(|_, _, cx| {
                                    cx.toast("Your changes have been saved successfully")
                                        .title("Success!")
                                        .success()
                                        .show();
                                }),
                            ),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .child(
                                button("warning", "Warning").on_click(|_, _, cx| {
                                    cx.toast("Please review your input before submitting")
                                        .title("Warning")
                                        .warning()
                                        .show();
                                }),
                            )
                            .child(
                                button("error", "Error").on_click(|_, _, cx| {
                                    cx.toast("Failed to save your changes. Please try again.")
                                        .title("Error")
                                        .error()
                                        .show();
                                }),
                            ),
                    ),
            )
            // Action buttons section
            .child(
                div()
                    .mt_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.fg_muted())
                            .child("With action button:"),
                    )
                    .child(
                        button("with-action", "Toast with Action").on_click(|_, _, cx| {
                            cx.toast("File deleted")
                                .warning()
                                .action("Undo", |_event, _window, _cx| {
                                    println!("Undo clicked!");
                                })
                                .duration(Duration::from_secs(6))
                                .show();
                        }),
                    ),
            )
            // Position controls
            .child(
                div()
                    .mt_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.fg_muted())
                            .child("Change position:"),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .child(
                                button("pos-tr", "Top Right").on_click(|_, _, cx| {
                                    cx.set_toast_position(ToastPosition::TopRight);
                                    cx.toast("Position: Top Right").info().show();
                                }),
                            )
                            .child(
                                button("pos-tl", "Top Left").on_click(|_, _, cx| {
                                    cx.set_toast_position(ToastPosition::TopLeft);
                                    cx.toast("Position: Top Left").info().show();
                                }),
                            )
                            .child(
                                button("pos-br", "Bottom Right").on_click(|_, _, cx| {
                                    cx.set_toast_position(ToastPosition::BottomRight);
                                    cx.toast("Position: Bottom Right").info().show();
                                }),
                            )
                            .child(
                                button("pos-bl", "Bottom Left").on_click(|_, _, cx| {
                                    cx.set_toast_position(ToastPosition::BottomLeft);
                                    cx.toast("Position: Bottom Left").info().show();
                                }),
                            ),
                    ),
            )
            // Toast container - renders toasts as deferred overlay
            .child(toast_container(cx))
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            let bounds = Bounds::centered(None, size(px(900.), px(700.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| ToastExample::new(window, cx));
                    let focus_handle = view.read(cx).focus_handle.clone();
                    window.focus(&focus_handle, cx);
                    view
                },
            )
            .unwrap();

            cx.activate(true);
        });
}
