//! Example demonstrating the TextInput component from gpui_input crate.

use gpui::{
    div, px, size, AppContext, Application, Bounds, Context, Entity, IntoElement, ParentElement,
    Render, Styled, VisualContext, WindowBounds, WindowOptions,
};
use gpui_input::TextInput;
use std::sync::Arc;
use theme::{GlobalTheme, Theme};

/// Main view containing the input example
struct InputExample {
    text_input: Entity<TextInput>,
    text_input_disabled: Entity<TextInput>,
}

impl InputExample {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create an enabled text input
        let text_input = cx.new(|cx| {
            TextInput::new("text-input", "Hello, GPUI!", cx)
                .placeholder("Enter some text...")
                .width(300.0)
        });

        // Create a disabled text input
        let text_input_disabled = cx.new(|cx| {
            TextInput::new("text-input-disabled", "This is disabled", cx)
                .placeholder("You can't edit this...")
                .width(300.0)
                .disabled(true)
        });

        Self {
            text_input,
            text_input_disabled,
        }
    }
}

impl Render for InputExample {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<GlobalTheme>().0.clone();

        // Read current value directly from input
        let text_value = self.text_input.read(cx).get_value().to_string();

        div()
            .flex()
            .flex_col()
            .gap(px(32.))
            .p(px(40.))
            .bg(theme.bg)
            .size_full()
            // Enabled Text Input Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_color(theme.fg)
                            .text_size(px(16.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Text Input"),
                    )
                    .child(self.text_input.clone())
                    .child(
                        div()
                            .text_color(theme.fg.alpha(0.6))
                            .text_size(px(12.))
                            .child(format!("Current value: \"{}\"", text_value)),
                    ),
            )
            // Disabled Text Input Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_color(theme.fg)
                            .text_size(px(16.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Disabled Input"),
                    )
                    .child(self.text_input_disabled.clone())
                    .child(
                        div()
                            .text_color(theme.fg.alpha(0.6))
                            .text_size(px(12.))
                            .child("This input cannot be edited"),
                    ),
            )
    }
}

fn main() {
    env_logger::init();

    let app = Application::new();

    app.run(move |cx| {
        // Set up the theme
        cx.set_global(GlobalTheme(Arc::new(Theme::gruvbox_dark())));

        // Create and open the window
        let bounds = Bounds::centered(None, size(px(500.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(InputExample::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
