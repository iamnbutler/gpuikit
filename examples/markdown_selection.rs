//! Markdown text selection.
//!
//! Drag across paragraphs, headings, lists and code blocks; double-click a
//! word; triple-click a block; ⌘C copies the selection.
//!
//! The pattern to note: selection state lives on the `Markdown` entity, so
//! the entity must be *retained* (created once, held by your view). The
//! `markdown()` convenience creates a fresh entity per call and therefore
//! cannot hold a selection across frames.
#![allow(missing_docs)]

use gpui::{
    actions, div, prelude::*, px, size, App, Application, Bounds, ClipboardItem, Context, Entity,
    KeyBinding, Window, WindowBounds, WindowOptions,
};
use gpuikit::markdown::{Markdown, MarkdownElement, MarkdownStyle};
use gpuikit::theme::{ActiveTheme, Themeable};

actions!(markdown_selection_example, [CopySelection]);

const SOURCE: &str = "\
# Selection demo

Drag from this paragraph into the heading above or the list below — the
selection flows across blocks. A [link](https://example.com) still opens on a
plain click, and `inline code` selects like any other text.

- First item
- Second item with **bold** text
- Third item

```rust
fn main() {
    println!(\"code blocks select too\");
}
```

> Block quotes as well. Press cmd-c to copy whatever is highlighted.
";

struct Example {
    markdown: Entity<Markdown>,
}

impl Example {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            markdown: cx.new(|cx| Markdown::new(SOURCE, cx)),
        }
    }
}

impl Render for Example {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .size_full()
            .bg(theme.bg())
            .text_color(theme.fg())
            .overflow_hidden()
            .p(px(24.))
            .on_action(cx.listener(|this, _: &CopySelection, _window, cx| {
                if let Some(text) = this.markdown.read(cx).selected_text() {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            }))
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
        cx.bind_keys([KeyBinding::new("cmd-c", CopySelection, None)]);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(560.), px(640.)),
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
