use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, Entity, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, Menu, ParentElement, Render, StatefulInteractiveElement,
    Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpuikit::{
    elements::{
        avatar::avatar,
        button::button,
        dropdown::{Dropdown, DropdownChanged, DropdownState},
        slider::{Slider, SliderChanged},
        toggle::{Toggle, ToggleChanged},
    },
    layout::{h_stack, v_stack},
};
use gpuikit_markdown::{Markdown, MarkdownElement};
use gpuikit_theme::{self, ActiveTheme};

const EXAMPLE_MARKDOWN: &str = r#"# Markdown syntax guide

## Headers

# This is a Heading h1
## This is a Heading h2
###### This is a Heading h6

## Emphasis

*This text will be italic*
_This will also be italic_

**This text will be bold**
__This will also be bold__

~~This text will be strikethrough~~

_You **can** combine them_

Here is some **bold text** in the middle of a sentence.

This sentence has *italic*, **bold**, and ~~strikethrough~~ all together.

**Bold with *nested italic* inside** and *italic with **nested bold** inside*.

A paragraph with ~~strikethrough **bold strikethrough** and *italic strikethrough*~~ text.

## Lists

### Unordered

* Item 1
* Item 2
* Item 2a
* Item 2b
    * Item 3a
    * Item 3b

### Ordered

1. Item 1
2. Item 2
3. Item 3
    1. Item 3a
    2. Item 3b

## Images

![Zed Logo](https://zed.dev/img/logo.svg)

## Links

Check out [Zed](https://zed.dev) - a high-performance code editor.

You may also like [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).

## Blockquotes

> Markdown is a lightweight markup language with plain-text-formatting syntax, created in 2004 by John Gruber with Aaron Swartz.
>
>> Markdown is often used to format readme files, for writing messages in online discussion forums, and to create rich text using a plain text editor.

## Tables

| Left columns  | Right columns |
| ------------- |:-------------:|
| left foo      | right foo     |
| left bar      | right bar     |
| left baz      | right baz     |

## Blocks of code

```
let message = 'Hello world';
alert(message);
```

## Inline code

This web site is using `markedjs/marked`."#;

#[derive(Clone, PartialEq)]
enum ColorOption {
    Red,
    Green,
    Blue,
    Yellow,
}

struct Showcase {
    focus_handle: FocusHandle,
    click_count: usize,
    color_dropdown: Entity<DropdownState<ColorOption>>,
    slider: Entity<Slider>,
    slider_value: f32,
    selected_color: ColorOption,
    toggle: Entity<Toggle>,
    toggle_enabled: bool,
    markdown: Entity<Markdown>,
}

impl Showcase {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let color_options = vec![
            (ColorOption::Red, "Red"),
            (ColorOption::Green, "Green"),
            (ColorOption::Blue, "Blue"),
            (ColorOption::Yellow, "Yellow"),
        ];

        let color_dropdown = cx.new(|_cx| {
            DropdownState::new(Dropdown::new(
                "color-dropdown",
                color_options,
                ColorOption::Blue,
            ))
        });

        cx.subscribe(
            &color_dropdown,
            |this, _dropdown, _event: &DropdownChanged, cx| {
                this.selected_color = _dropdown.read(cx).selected.clone();
                cx.notify();
            },
        )
        .detach();

        let slider = cx.new(|_cx| {
            Slider::new("value-slider", 50.0, 0.0..=100.0)
                .label("Value")
                .step(1.0)
        });

        cx.subscribe(&slider, |this, _slider, event: &SliderChanged, cx| {
            this.slider_value = event.value;
            cx.notify();
        })
        .detach();

        let toggle = cx.new(|_cx| Toggle::new("feature-toggle", false).label("Enable feature"));

        cx.subscribe(&toggle, |this, _toggle, event: &ToggleChanged, cx| {
            this.toggle_enabled = event.enabled;
            cx.notify();
        })
        .detach();

        let markdown = cx.new(|cx| Markdown::new(EXAMPLE_MARKDOWN, cx));

        Self {
            focus_handle: cx.focus_handle(),
            click_count: 0,
            color_dropdown,
            slider,
            slider_value: 50.0,
            selected_color: ColorOption::Blue,
            toggle,
            toggle_enabled: false,
            markdown,
        }
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_stack()
            .gap_4()
            .p_8()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.fg)
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
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
                            .child(button("reset-btn", "Reset Counter").on_click(cx.listener(
                                |showcase, _event, _window, cx| {
                                    showcase.click_count = 0;
                                    cx.notify();
                                },
                            ))),
                    )
                    .child(
                        h_stack()
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
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Avatar"),
                    )
                    .child(h_stack().gap_2().child(
                        avatar("https://avatars.githubusercontent.com/u/1714999?v=4").size(px(32.)),
                    )),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Dropdown"),
                    )
                    .child(
                        h_stack()
                            .gap_4()
                            .items_center()
                            .child(div().w(px(150.)).child(self.color_dropdown.clone()))
                            .child(
                                h_stack().items_center().gap_2().child("Selected:").child(
                                    div()
                                        .text_color(theme.accent)
                                        .font_weight(FontWeight::BOLD)
                                        .child(match self.selected_color {
                                            ColorOption::Red => "Red",
                                            ColorOption::Green => "Green",
                                            ColorOption::Blue => "Blue",
                                            ColorOption::Yellow => "Yellow",
                                        }),
                                ),
                            ),
                    ),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Slider"),
                    )
                    .child(
                        h_stack()
                            .gap_4()
                            .items_center()
                            .child(div().w(px(200.)).child(self.slider.clone()))
                            .child(
                                h_stack()
                                    .items_center()
                                    .gap_2()
                                    .child("Current value:")
                                    .child(
                                        div()
                                            .text_color(theme.accent)
                                            .font_weight(FontWeight::BOLD)
                                            .child(format!("{:.0}", self.slider_value)),
                                    ),
                            ),
                    ),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Toggle"),
                    )
                    .child(
                        h_stack()
                            .gap_4()
                            .items_center()
                            .child(self.toggle.clone())
                            .child(
                                h_stack().items_center().gap_2().child("Status:").child(
                                    div()
                                        .text_color(if self.toggle_enabled {
                                            theme.accent
                                        } else {
                                            theme.fg_muted
                                        })
                                        .font_weight(FontWeight::BOLD)
                                        .child(if self.toggle_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }),
                                ),
                            ),
                    ),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.fg_muted)
                            .child("Markdown"),
                    )
                    .child(
                        div()
                            .id("markdown-container")
                            .p_4()
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.surface)
                            .max_h(px(400.0))
                            .overflow_y_scroll()
                            .child(MarkdownElement::new(self.markdown.clone())),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
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
                        size: size(px(800.0), px(600.0)),
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
