//! Streaming markdown: a reply arriving a few characters at a time.
//!
//! The pattern to note is `Markdown::append` — it extends the source and
//! re-parses on a background thread, keeping the previous parse on screen
//! until the new one lands. Deltas that arrive during a parse coalesce, so a
//! fast stream does not queue up one parse per token.
//!
//! Build with `--features stitch` to also close the syntax a half-written
//! document leaves open (`**bold` with no closer, `[label](htt`). Without it
//! the text visibly flickers between literal markers and styled text as each
//! delta arrives; the status line at the top says which build you are running.
//!
//! Press space to stream the reply again.
#![allow(missing_docs)]

use std::time::Duration;

use gpui::{
    actions, div, prelude::*, px, rems, size, App, Application, Bounds, Context, Entity,
    KeyBinding, Window, WindowBounds, WindowOptions,
};
use gpuikit::markdown::{preprocessing_available, Markdown, MarkdownElement, MarkdownStyle};
use gpuikit::theme::{ActiveTheme, Themeable};

actions!(markdown_streaming_example, [Restart]);

/// Roughly what a model's answer looks like coming off the wire.
const REPLY: &str = "\
# Streaming a reply

Here is a **bold claim** with a [link](https://example.com) in it, plus some
`inline code`, arriving a few characters at a time — the same shape an LLM
response has.

- A list item that is long enough to wrap onto a second line inside this window
- One with *emphasis* in the middle of it
- And a ~~struck out~~ one

```rust
fn main() {
    println!(\"code fences stream too\");
}
```

> Every delta goes in through `Markdown::append`, which re-parses off the UI
> thread and keeps the previous parse on screen until the new one lands.
";

/// Characters per delta and the gap between them — about a frame apart, which
/// is faster than any real stream and so a fair bit of pressure on the parse.
const CHUNK: usize = 3;
const INTERVAL: Duration = Duration::from_millis(16);

struct Example {
    markdown: Entity<Markdown>,
    /// Bumped on restart so an in-flight stream stops feeding a document that
    /// has already been reset.
    generation: usize,
}

impl Example {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            markdown: cx.new(|cx| Markdown::new("", cx)),
            generation: 0,
        };
        this.restart(cx);
        this
    }

    fn restart(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.markdown
            .update(cx, |markdown, cx| markdown.set_source("", cx));

        cx.spawn(async move |this, cx| {
            let mut sent = 0;
            while sent < REPLY.len() {
                cx.background_executor().timer(INTERVAL).await;

                // Deltas land on character boundaries; a real stream would
                // hand over whole tokens.
                let mut end = (sent + CHUNK).min(REPLY.len());
                while !REPLY.is_char_boundary(end) {
                    end += 1;
                }
                let delta = &REPLY[sent..end];
                sent = end;

                let still_current = this.update(cx, |this, cx| {
                    if this.generation != generation {
                        return false;
                    }
                    this.markdown
                        .update(cx, |markdown, cx| markdown.append(delta, cx));
                    true
                });

                match still_current {
                    Ok(true) => {}
                    // Restarted, or the view is gone.
                    Ok(false) | Err(_) => break,
                }
            }
        })
        .detach();

        cx.notify();
    }
}

impl Render for Example {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let status = if preprocessing_available() {
            "partial syntax is closed before parsing (built with --features stitch)"
        } else {
            "no preprocessing — rebuild with --features stitch to stop the flicker"
        };

        div()
            .size_full()
            .bg(theme.bg())
            .text_color(theme.fg())
            .overflow_hidden()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .on_action(cx.listener(|this, _: &Restart, _window, cx| this.restart(cx)))
            .child(
                div()
                    .text_size(rems(0.75))
                    .text_color(theme.fg_muted())
                    .child(format!("{status} · space to restart")),
            )
            .child(
                MarkdownElement::new(self.markdown.clone())
                    .style(MarkdownStyle::new().soft_break_as_hard_break(true)),
            )
    }
}

fn main() {
    let app = Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets());
    app.run(|cx: &mut App| {
        gpuikit::theme::init(cx);
        cx.bind_keys([KeyBinding::new("space", Restart, None)]);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(560.), px(720.)),
                    cx,
                ))),
                ..Default::default()
            },
            |_, cx| cx.new(Example::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
