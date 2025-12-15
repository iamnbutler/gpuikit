use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, Entity, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, Menu, ParentElement, Render, StatefulInteractiveElement,
    Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpuikit::markdown::{Markdown, MarkdownElement};
use gpuikit::theme::{ActiveTheme, Themeable};
use gpuikit::{
    elements::{
        avatar::avatar,
        button::button,
        dropdown::{dropdown, DropdownState},
        icon_button::icon_button,
        separator::{separator, vertical_separator},
    },
    layout::{h_stack, v_stack},
    DefaultIcons,
};

const SAMPLE_MARKDOWN: &str = r#"# Markdown Showcase

This is a **bold** statement and this is *italic*.

## Features

- Bullet lists
- **Bold** and *italic* text
- `inline code`

### Code Blocks

```rust
fn main() {
    println!("Hello, GPUI!");
}
```

### Blockquotes

> This is a blockquote.
> It can span multiple lines.

### Links & More

Visit [GPUI](https://zed.dev) for more info.

---

1. Numbered lists
2. Work too
3. Like this

| Column 1 | Column 2 |
|----------|----------|
| Cell A   | Cell B   |
| Cell C   | Cell D   |
"#;

#[derive(Clone, PartialEq, Debug)]
enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, PartialEq, Debug)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

struct Showcase {
    focus_handle: FocusHandle,
    click_count: usize,
    toggled_count: usize,
    size_dropdown: Entity<DropdownState<Size>>,
    priority_dropdown: Entity<DropdownState<Priority>>,
    markdown: Entity<Markdown>,
}

impl Showcase {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let size_dropdown = cx.new(|_cx| {
            DropdownState::new(dropdown(
                "size-dropdown",
                vec![
                    (Size::Small, "Small"),
                    (Size::Medium, "Medium"),
                    (Size::Large, "Large"),
                ],
                Size::Medium,
            ))
        });

        let priority_dropdown = cx.new(|_cx| {
            DropdownState::new(dropdown(
                "priority-dropdown",
                vec![
                    (Priority::Low, "Low"),
                    (Priority::Normal, "Normal"),
                    (Priority::High, "High"),
                    (Priority::Critical, "Critical"),
                ],
                Priority::Normal,
            ))
        });

        let markdown = cx.new(|cx| Markdown::new(SAMPLE_MARKDOWN, cx));

        Self {
            focus_handle: cx.focus_handle(),
            click_count: 0,
            toggled_count: 0,
            size_dropdown,
            priority_dropdown,
            markdown,
        }
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_stack()
            .bg(theme.bg())
            .text_color(theme.fg())
            .size_full()
            .overflow_hidden()
            .child(
                v_stack()
                    .gap_4()
                    .p_8()
                    .h_full()
                    .flex_1()
                    .child(
                        v_stack()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.fg_muted())
                                    .child("Button"),
                            )
                            .child(
                                h_stack()
                                    .gap_2()
                                    .child(button("click-me", "Click Me").on_click(cx.listener(
                                        |showcase, _event, _window, cx| {
                                            showcase.click_count += 1;
                                            cx.notify();
                                        },
                                    )))
                                    .child(button("disabled-btn", "Disabled Button").disabled(true))
                                    .child(button("reset-btn", "Reset Counter").on_click(
                                        cx.listener(|showcase, _event, _window, cx| {
                                            showcase.click_count = 0;
                                            cx.notify();
                                        }),
                                    )),
                            )
                            .child(
                                h_stack()
                                    .items_center()
                                    .gap_2()
                                    .mt_2()
                                    .child("Click count:")
                                    .child(
                                        div()
                                            .text_color(theme.accent())
                                            .font_weight(FontWeight::BOLD)
                                            .child(format!("{}", self.click_count)),
                                    ),
                            ),
                    )
                    .child(separator())
                    .child(
                        v_stack()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.fg_muted())
                                    .child("Icon Button"),
                            )
                            .child(
                                h_stack()
                                    .gap_2()
                                    .items_center()
                                    .child(icon_button("icon-star", DefaultIcons::star()))
                                    .child(icon_button("icon-heart", DefaultIcons::heart()))
                                    .child(icon_button("icon-gear", DefaultIcons::gear()))
                                    .child(icon_button("icon-bell", DefaultIcons::bell()))
                                    .child(icon_button("icon-home", DefaultIcons::home()))
                                    .child(icon_button(
                                        "icon-search",
                                        DefaultIcons::magnifying_glass(),
                                    ))
                                    .child(icon_button("icon-plus", DefaultIcons::plus()))
                                    .child(icon_button("icon-trash", DefaultIcons::trash())),
                            )
                            .child(
                                h_stack()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        icon_button("icon-selected", DefaultIcons::check_circled())
                                            .selected(true),
                                    )
                                    .child(
                                        icon_button("icon-disabled", DefaultIcons::lock_closed())
                                            .disabled(true),
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.fg_muted())
                                            .child("(selected / disabled)"),
                                    ),
                            )
                            .child(
                                h_stack()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        icon_button("toggle-star", DefaultIcons::star())
                                            .use_state()
                                            .on_toggle(cx.listener(
                                                |showcase, toggled, _window, cx| {
                                                    if *toggled {
                                                        showcase.toggled_count += 1;
                                                    } else {
                                                        showcase.toggled_count = showcase
                                                            .toggled_count
                                                            .saturating_sub(1);
                                                    }
                                                    cx.notify();
                                                },
                                            )),
                                    )
                                    .child(
                                        icon_button("toggle-heart", DefaultIcons::heart())
                                            .use_state()
                                            .on_toggle(cx.listener(
                                                |showcase, toggled, _window, cx| {
                                                    if *toggled {
                                                        showcase.toggled_count += 1;
                                                    } else {
                                                        showcase.toggled_count = showcase
                                                            .toggled_count
                                                            .saturating_sub(1);
                                                    }
                                                    cx.notify();
                                                },
                                            )),
                                    )
                                    .child(
                                        icon_button("toggle-bell", DefaultIcons::bell())
                                            .use_state()
                                            .on_toggle(cx.listener(
                                                |showcase, toggled, _window, cx| {
                                                    if *toggled {
                                                        showcase.toggled_count += 1;
                                                    } else {
                                                        showcase.toggled_count = showcase
                                                            .toggled_count
                                                            .saturating_sub(1);
                                                    }
                                                    cx.notify();
                                                },
                                            )),
                                    )
                                    .child(
                                        h_stack()
                                            .gap_2()
                                            .items_center()
                                            .text_color(theme.fg_muted())
                                            .child("Toggled:")
                                            .child(
                                                div()
                                                    .text_color(theme.accent())
                                                    .font_weight(FontWeight::BOLD)
                                                    .child(format!("{}", self.toggled_count)),
                                            ),
                                    ),
                            ),
                    )
                    .child(separator())
                    .child(
                        v_stack()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.fg_muted())
                                    .child("Dropdown"),
                            )
                            .child(
                                h_stack()
                                    .gap_4()
                                    .items_start()
                                    .child(
                                        v_stack()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.fg_muted())
                                                    .child("Size"),
                                            )
                                            .child(self.size_dropdown.clone()),
                                    )
                                    .child(
                                        v_stack()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.fg_muted())
                                                    .child("Priority"),
                                            )
                                            .child(self.priority_dropdown.clone()),
                                    ),
                            )
                            .child(
                                h_stack()
                                    .gap_4()
                                    .items_center()
                                    .mt_2()
                                    .child(
                                        h_stack()
                                            .gap_2()
                                            .items_center()
                                            .text_color(theme.fg_muted())
                                            .child("Selected size:")
                                            .child(
                                                div()
                                                    .text_color(theme.accent())
                                                    .font_weight(FontWeight::BOLD)
                                                    .child(format!(
                                                        "{:?}",
                                                        self.size_dropdown.read(cx).selected
                                                    )),
                                            ),
                                    )
                                    .child(
                                        h_stack()
                                            .gap_2()
                                            .items_center()
                                            .text_color(theme.fg_muted())
                                            .child("Selected priority:")
                                            .child(
                                                div()
                                                    .text_color(theme.accent())
                                                    .font_weight(FontWeight::BOLD)
                                                    .child(format!(
                                                        "{:?}",
                                                        self.priority_dropdown.read(cx).selected
                                                    )),
                                            ),
                                    ),
                            ),
                    )
                    .child(separator())
                    .child(
                        v_stack()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.fg_muted())
                                    .child("Avatar"),
                            )
                            .child(
                                h_stack().gap_2().child(
                                    avatar("https://avatars.githubusercontent.com/u/1714999?v=4")
                                        .size(px(32.)),
                                ),
                            ),
                    ),
            )
            .child(vertical_separator())
            .child(
                v_stack()
                    .id("markdown-panel")
                    .gap_4()
                    .p_8()
                    .overflow_y_scroll()
                    .min_h_full()
                    .flex_1()
                    .child(MarkdownElement::new(self.markdown.clone())),
            )
    }
}

fn main() {
    Application::new()
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            cx.set_menus(vec![Menu {
                name: "GPUIKit Showcase".into(),
                items: vec![],
            }]);

            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: Some(TitlebarOptions {
                            title: Some("GPUIKit Component Showcase".into()),
                            ..Default::default()
                        }),
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: Default::default(),
                            size: size(px(1200.0), px(680.0)),
                        })),
                        ..Default::default()
                    },
                    |window, cx| cx.new(|cx| Showcase::new(window, cx)),
                )
                .unwrap();

            window
                .update(cx, |showcase, window, cx| {
                    window.focus(&showcase.focus_handle);
                    cx.activate(true);
                })
                .unwrap();
        });
}
