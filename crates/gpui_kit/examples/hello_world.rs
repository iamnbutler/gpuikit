use std::sync::Arc;

use assets::{
    icons::{Icon, IconName},
    Assets,
};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};
use theme::{ActiveTheme, GlobalTheme, Theme};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(theme.bg)
            .size(px(500.0))
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(theme.fg)
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(Icon::new(IconName::Check, theme.fg))
                    .child(Icon::new(IconName::QuestionMark, theme.fg)),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        cx.set_global(GlobalTheme(Arc::new(Theme::default())));

        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "GPUI Kit".into(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
