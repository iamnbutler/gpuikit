use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, ElementId, FocusHandle,
    FontWeight, IntoElement, Menu, ParentElement, Render, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions,
};
use gpuikit::elements::button::button;
use gpuikit_theme::{self, ActiveTheme};

struct Showcase {
    focus_handle: FocusHandle,
    click_count: usize,
}

impl Showcase {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            click_count: 0,
        }
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.fg)
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.fg)
                    .child("GPUIKit Component Showcase"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Button Examples"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(button("click-me", "Click Me").on_click(cx.listener(
                                |showcase, _event, _window, cx| {
                                    showcase.click_count += 1;
                                    cx.notify();
                                },
                            )))
                            .child(button("disabled-btn", "Disabled Button").disabled(true))
                            .child(button("reset-btn", "Reset Counter").on_click(cx.listener(
                                |showcase, _event, _window, cx| {
                                    showcase.click_count = 0;
                                    cx.notify();
                                },
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .mt_2()
                            .child("Click count:")
                            .child(
                                div()
                                    .text_color(theme.accent)
                                    .font_weight(FontWeight::BOLD)
                                    .child(format!("{}", self.click_count)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .mt_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Dynamic Button Creation"),
                    )
                    .child(div().flex().gap_2().children((0..5).map(|i| {
                        button(
                            ElementId::Name(format!("btn-{}", i + 1).into()),
                            format!("Button {}", i + 1),
                        )
                        .on_click({
                            let index = i + 1;
                            move |_event, _window, _cx| {
                                println!("Dynamic button {} clicked", index);
                            }
                        })
                    }))),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Initialize global systems
        gpuikit::init(cx);

        // Set up the application menu
        cx.set_menus(vec![Menu {
            name: "GPUIKit Showcase".into(),
            items: vec![],
        }]);

        // Open the main window
        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("GPUIKit Component Showcase".into()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Default::default(),
                        size: size(px(800.0), px(600.0)),
                    })),
                    ..Default::default()
                },
                |_window, cx| cx.new(Showcase::new),
            )
            .unwrap();

        // Focus the window
        window
            .update(cx, |showcase, window, cx| {
                window.focus(&showcase.focus_handle);
                cx.activate(true);
            })
            .unwrap();
    });
}
