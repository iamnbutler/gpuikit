//! Popover demo - demonstrates the Popover component.
//!
//! Run with: cargo run --example popover_demo

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle, Focusable,
    IntoElement, ParentElement, Render, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_platform;
use gpuikit::elements::button::button;
use gpuikit::elements::popover::{popover, PopoverState};
use gpuikit::layout::v_stack;
use gpuikit::theme::{ActiveTheme, Themeable};

struct PopoverDemo {
    focus_handle: FocusHandle,
    basic_popover: Entity<PopoverState>,
    content_popover: Entity<PopoverState>,
}

impl PopoverDemo {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let basic_popover = cx.new(|_cx| {
            PopoverState::new(
                popover("basic-popover")
                    .trigger(|_window, _cx| {
                        button("trigger-basic", "Open Popover").into_any_element()
                    })
                    .content(|_window, _cx| {
                        v_stack()
                            .p_3()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Basic Popover"),
                            )
                            .child(div().text_xs().child("Click outside or press Escape to close."))
                            .into_any_element()
                    }),
            )
        });

        let content_popover = cx.new(|_cx| {
            PopoverState::new(
                popover("content-popover")
                    .trigger(|_window, _cx| {
                        button("trigger-content", "Rich Content").into_any_element()
                    })
                    .content(|_window, cx| {
                        let theme = cx.theme();
                        v_stack()
                            .p_4()
                            .gap_3()
                            .w(px(250.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Settings"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Popovers can contain any content."),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded_md()
                                    .bg(theme.surface_secondary())
                                    .text_xs()
                                    .child("Nested content area"),
                            )
                            .into_any_element()
                    }),
            )
        });

        Self {
            focus_handle: cx.focus_handle(),
            basic_popover,
            content_popover,
        }
    }
}

impl Focusable for PopoverDemo {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("popover-demo")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(theme.bg())
            .text_color(theme.fg())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_8()
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Popover Demo"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.fg_muted())
                    .child("Click buttons to toggle. Click outside or Escape to close."),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(self.basic_popover.clone())
                    .child(self.content_popover.clone()),
            )
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            let bounds = Bounds::centered(None, size(px(600.), px(400.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| PopoverDemo::new(window, cx)),
            )
            .unwrap();
        });
}
