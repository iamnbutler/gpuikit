//! Example demonstrating the TextInput and NumericInput components from gpui_input crate.

use gpui::{
    div, px, size, AppContext, Application, Bounds, Context, Entity, IntoElement, ParentElement,
    Render, Styled, WindowBounds, WindowOptions,
};
use gpui_input::{NumericInput, TextInput};
use std::sync::Arc;
use theme::{GlobalTheme, Theme};

/// Main view containing the input examples
struct InputExample {
    text_input: Entity<TextInput>,
    numeric_input: Entity<NumericInput>,
}

impl InputExample {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create the text input entity
        let text_input = cx.new(|cx| {
            TextInput::new("text-input", "", cx)
                .placeholder("Enter some text...")
                .width(250.0)
        });

        // Create the numeric input entity
        let numeric_input = cx.new(|cx| {
            NumericInput::new("numeric-input", Some(42.0), cx)
                .placeholder("Enter a number...")
                .min(0.0)
                .max(100.0)
                .width(150.0)
        });

        Self {
            text_input,
            numeric_input,
        }
    }
}

impl Render for InputExample {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<GlobalTheme>().0.clone();

        // Read current values directly from inputs
        let text_value = self.text_input.read(cx).get_value().to_string();
        let numeric_value = self.numeric_input.read(cx).get_value();

        div()
            .flex()
            .flex_col()
            .gap(px(32.))
            .p(px(40.))
            .bg(theme.bg)
            .size_full()
            // Text Input Section
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
            // Numeric Input Section
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
                            .child("Numeric Input (0-100)"),
                    )
                    .child(self.numeric_input.clone())
                    .child(
                        div()
                            .text_color(theme.fg.alpha(0.6))
                            .text_size(px(12.))
                            .child(format!(
                                "Current value: {}",
                                numeric_value
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| "None".to_string())
                            )),
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
        let bounds = Bounds::centered(None, size(px(600.0), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(InputExample::new),
        )
        .unwrap();
    });
}
