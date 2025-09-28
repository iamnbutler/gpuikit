use assets::{
    icons::{Icon, IconName},
    Assets,
};
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(Icon::new(IconName::Check))
                    .child(Icon::new(IconName::QuestionMark)),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
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

// use gpui::{
//     div, point, prelude::*, px, rgb, size, App, Bounds, GlobalPixels, SharedString, Size,
//     WindowBounds, WindowOptions,
// };

// mod prelude;

// use prelude::*;

// struct Hello {
//     text: SharedString,
// }

// impl Render for Hello {
//     fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
//         div()
//             .flex()
//             .bg(rgb(0x000000))
//             .size_full()
//             .justify_center()
//             .items_center()
//             .text_xl()
//             .text_color(rgb(0xffffff))
//             .child(icons::Icon::new(IconName::Check))
//             .child(icons::Icon::new(IconName::QuestionMark))
//             .child(format!("Hello, {}!", &self.text))
//     }
// }

// fn main() {
//     App::new().with_assets(Assets).run(|cx| {
//         let displays = cx.displays();
//         let first_display = displays.first().expect("no displays");

//         let window_size: Size<GlobalPixels> = size(px(800.), px(600.)).into();
//         let window_origin = point(
//             first_display.bounds().center().x - window_size.width / 2.,
//             first_display.bounds().center().y - window_size.height / 2.,
//         );

//         cx.open_window(
//             WindowOptions {
//                 bounds: WindowBounds::Fixed(Bounds::<GlobalPixels>::new(
//                     window_origin,
//                     size(px(800.), px(600.)).into(),
//                 )),
//                 ..Default::default()
//             },
//             |cx| {
//                 cx.new_view(|_cx| Hello {
//                     text: "GPUI Kit".into(),
//                 })
//             },
//         );
//     })
// }
