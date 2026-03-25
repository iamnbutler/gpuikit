#![allow(missing_docs)]
use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, Entity, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, Menu, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpui_platform;
use gpuikit::input::InputState;
use gpuikit::markdown::{Markdown, MarkdownElement};
use gpuikit::theme::{ActiveTheme, Themeable};
use gpuikit::{
    elements::{
        accordion::{accordion, accordion_item, AccordionState},
        alert::alert,
        aspect_ratio::{aspect_ratio, aspect_ratio_square, aspect_ratio_video},
        avatar::avatar,
        badge::badge,
        breadcrumb::{breadcrumb, breadcrumb_item, BreadcrumbSeparator},
        button::button,
        button_group::button_group,
        card::card,
        checkbox::{checkbox, Checkbox},
        collapsible::{collapsible, Collapsible},
        context_menu::{context_menu, menu_item, menu_separator, ContextMenuState},
        dialog::{dialog, DialogState},
        dropdown::{dropdown, DropdownState},
        field::{field, LabelPosition},
        icon_button::icon_button,
        input_group::{input_group, InputAddon},
        kbd::{kbd, kbd_combo, KbdSize},
        label::label,
        list::{List, ListEntry},
        loading_indicator::loading_indicator,
        popover::{popover, PopoverState},
        progress::{progress, ProgressVariant},
        radio_group::{radio_group, radio_option, RadioGroup},
        scroll_area::scroll_area,
        select::{select, SelectState},
        separator::separator,
        skeleton::{skeleton, skeleton_avatar, skeleton_card, skeleton_text},
        switch::{switch, Switch},
        tabs::{tab, tabs, Tabs},
        textarea::textarea,
        toast::ToastExt,
        toggle_group::{toggle_group, toggle_option, ToggleGroup, ToggleGroupMode},
        tooltip::tooltip,
    },
    layout::{h_stack, v_stack},
    traits::disableable::Disableable,
    traits::labelable::Labelable,
    traits::orientable::Orientable,
    DefaultIcons,
};
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Clone, PartialEq, Debug)]
enum NotificationPreference {
    All,
    Important,
    None,
}

#[derive(Clone, PartialEq, Debug)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
enum TextStyle {
    Bold,
    Italic,
    Underline,
}

#[derive(Clone, PartialEq, Debug)]
enum Country {
    US,
    UK,
    CA,
    DE,
    FR,
}

struct Showcase {
    focus_handle: FocusHandle,
    active_page: Rc<RefCell<SharedString>>,
    click_count: usize,
    toggled_count: usize,
    size_dropdown: Entity<DropdownState<Size>>,
    priority_dropdown: Entity<DropdownState<Priority>>,
    country_select: Entity<SelectState<Country>>,
    markdown: Entity<Markdown>,
    checkbox_agree: Entity<Checkbox>,
    checkbox_newsletter: Entity<Checkbox>,
    radio_notifications: Entity<RadioGroup<NotificationPreference>>,
    switch_wifi: Entity<Switch>,
    switch_bluetooth: Entity<Switch>,
    switch_airplane: Entity<Switch>,
    collapsible_basic: Entity<Collapsible>,
    collapsible_nested: Entity<Collapsible>,
    accordion: Entity<AccordionState>,
    toggle_group_alignment: Entity<ToggleGroup<Alignment>>,
    toggle_group_text_style: Entity<ToggleGroup<TextStyle>>,
    tabs_example: Entity<Tabs>,
    input_with_icon: Entity<InputState>,
    input_with_text: Entity<InputState>,
    input_with_button: Entity<InputState>,
    textarea_example: Entity<InputState>,
    popover_example: Entity<PopoverState>,
    dialog_example: Entity<DialogState>,
    context_menu_example: Entity<ContextMenuState>,
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

        let country_select = cx.new(|_cx| {
            SelectState::new(
                select(
                    "country-select",
                    vec![
                        (Country::US, "United States"),
                        (Country::UK, "United Kingdom"),
                        (Country::CA, "Canada"),
                        (Country::DE, "Germany"),
                        (Country::FR, "France"),
                    ],
                )
                .placeholder("Choose a country..."),
            )
        });

        let markdown = cx.new(|cx| Markdown::new(SAMPLE_MARKDOWN, cx));

        let checkbox_agree =
            cx.new(|_cx| checkbox("agree-terms", false).label("I agree to the terms"));
        let checkbox_newsletter =
            cx.new(|_cx| checkbox("newsletter", true).label("Subscribe to newsletter"));

        let radio_notifications = cx.new(|_cx| {
            radio_group(
                "notifications",
                vec![
                    radio_option(NotificationPreference::All, "All notifications"),
                    radio_option(NotificationPreference::Important, "Important only"),
                    radio_option(NotificationPreference::None, "None"),
                ],
            )
            .selected(NotificationPreference::Important)
        });

        let switch_wifi = cx.new(|_cx| switch("wifi-switch", true).label("Wi-Fi"));
        let switch_bluetooth = cx.new(|_cx| switch("bluetooth-switch", false).label("Bluetooth"));
        let switch_airplane =
            cx.new(|_cx| switch("airplane-switch", false).label("Airplane Mode").disabled(true));

        let collapsible_basic = cx.new(|_cx| {
            collapsible("collapsible-basic")
                .trigger_label("Click to expand")
                .content(|_window, _cx| {
                    div()
                        .text_sm()
                        .child(
                            "This is the collapsible content. It can contain any elements you want.",
                        )
                        .into_any_element()
                })
                .default_open(false)
        });

        let collapsible_nested = cx.new(|_cx| {
            collapsible("collapsible-nested")
                .trigger_label("Settings")
                .content(|_window, _cx| {
                    v_stack()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .child("Configure your preferences below:"),
                        )
                        .child(
                            h_stack()
                                .gap_2()
                                .child(badge("Option 1"))
                                .child(badge("Option 2"))
                                .child(badge("Option 3")),
                        )
                        .into_any_element()
                })
                .default_open(true)
        });

        let accordion = cx.new(|_cx| {
            AccordionState::new(
                accordion("showcase-accordion")
                    .item(
                        accordion_item("getting-started", "Getting Started")
                            .content("Welcome to GPUIKit! This library provides a comprehensive set of UI components for building GPUI applications."),
                    )
                    .item(
                        accordion_item("installation", "Installation")
                            .content("Add gpuikit to your Cargo.toml and call gpuikit::init(cx) in your application."),
                    )
                    .item(
                        accordion_item("theming", "Theming")
                            .content("GPUIKit supports theming through the theme module. You can customize colors, fonts, and spacing."),
                    )
                    .item(
                        accordion_item("disabled-section", "Disabled Section")
                            .content("This section is disabled.")
                            .disabled(true),
                    )
                    .default_expanded("getting-started"),
            )
        });

        let toggle_group_alignment = cx.new(|_cx| {
            toggle_group(
                "alignment",
                vec![
                    toggle_option(Alignment::Left, "Left"),
                    toggle_option(Alignment::Center, "Center"),
                    toggle_option(Alignment::Right, "Right"),
                ],
            )
            .selected_value(Alignment::Center)
        });

        let toggle_group_text_style = cx.new(|_cx| {
            toggle_group(
                "text-style",
                vec![
                    toggle_option(TextStyle::Bold, "B"),
                    toggle_option(TextStyle::Italic, "I"),
                    toggle_option(TextStyle::Underline, "U"),
                ],
            )
            .mode(ToggleGroupMode::Multiple)
            .selected(vec![TextStyle::Bold])
        });

        let tabs_example = cx.new(|_cx| {
            tabs("example-tabs")
                .tab(tab("home", "Home"))
                .tab(tab("profile", "Profile"))
                .tab(tab("settings", "Settings"))
                .tab(tab("disabled", "Disabled").disabled(true))
        });

        let input_with_icon = cx.new(|cx| InputState::new_singleline(cx));
        let input_with_text = cx.new(|cx| InputState::new_singleline(cx));
        let input_with_button = cx.new(|cx| InputState::new_singleline(cx));
        let textarea_example = cx.new(|cx| InputState::new_multiline(cx));

        let popover_example = cx.new(|_cx| {
            PopoverState::new(
                popover("showcase-popover")
                    .trigger(|_window, _cx| {
                        button("popover-trigger", "Open Popover").into_any_element()
                    })
                    .content(|_window, cx| {
                        let theme = cx.theme();
                        v_stack()
                            .p_3()
                            .gap_2()
                            .w(px(200.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Popover Content"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Click outside or press Escape to close."),
                            )
                            .into_any_element()
                    }),
            )
        });

        let dialog_example = cx.new(|_cx| {
            DialogState::new(
                dialog("showcase-dialog")
                    .title("Confirm Action")
                    .description(
                        "Are you sure you want to proceed? This action cannot be undone.",
                    )
                    .footer(|_window, _cx| {
                        h_stack()
                            .gap_2()
                            .justify_end()
                            .child(button("dialog-cancel", "Cancel"))
                            .child(button("dialog-confirm", "Confirm"))
                            .into_any_element()
                    }),
            )
        });

        let context_menu_example = cx.new(|_cx| {
            ContextMenuState::new(
                context_menu("showcase-context-menu")
                    .trigger(|_window, cx| {
                        let theme = cx.theme();
                        div()
                            .px_4()
                            .py_3()
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border())
                            .bg(theme.surface())
                            .text_sm()
                            .text_color(theme.fg_muted())
                            .child("Right-click here")
                            .into_any_element()
                    })
                    .menu(|_window, _cx| {
                        vec![
                            menu_item("cut", "Cut").kbd("Cmd+X").into(),
                            menu_item("copy", "Copy").kbd("Cmd+C").into(),
                            menu_item("paste", "Paste").kbd("Cmd+V").into(),
                            menu_separator().into(),
                            menu_item("delete", "Delete").destructive().into(),
                        ]
                    }),
            )
        });

        Self {
            focus_handle: cx.focus_handle(),
            active_page: Rc::new(RefCell::new(SharedString::from("Button"))),
            click_count: 0,
            toggled_count: 0,
            size_dropdown,
            priority_dropdown,
            country_select,
            markdown,
            checkbox_agree,
            checkbox_newsletter,
            radio_notifications,
            switch_wifi,
            switch_bluetooth,
            switch_airplane,
            collapsible_basic,
            collapsible_nested,
            accordion,
            toggle_group_alignment,
            toggle_group_text_style,
            tabs_example,
            input_with_icon,
            input_with_text,
            input_with_button,
            textarea_example,
            popover_example,
            dialog_example,
            context_menu_example,
        }
    }

    fn render_button_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
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
            )
    }

    fn render_button_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ButtonGroup"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(
                        button_group("btn-group-1")
                            .child(button("group-1-a", "Left"))
                            .child(button("group-1-b", "Center"))
                            .child(button("group-1-c", "Right")),
                    )
                    .child(
                        button_group("btn-group-2")
                            .vertical()
                            .child(button("group-2-a", "Top"))
                            .child(button("group-2-b", "Middle"))
                            .child(button("group-2-c", "Bottom")),
                    ),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .mt_2()
                    .child(
                        div()
                            .text_color(theme.fg_muted())
                            .child("(horizontal / vertical)"),
                    ),
            )
    }

    fn render_icon_button_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
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
            )
    }

    fn render_checkbox_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Checkbox"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.checkbox_agree.clone())
                    .child(self.checkbox_newsletter.clone()),
            )
    }

    fn render_switch_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Switch"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.switch_wifi.clone())
                    .child(self.switch_bluetooth.clone())
                    .child(self.switch_airplane.clone()),
            )
    }

    fn render_radio_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("RadioGroup"),
            )
            .child(self.radio_notifications.clone())
    }

    fn render_toggle_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ToggleGroup"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .w_20()
                                    .child("Single:"),
                            )
                            .child(self.toggle_group_alignment.clone()),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .w_20()
                                    .child("Multiple:"),
                            )
                            .child(self.toggle_group_text_style.clone()),
                    ),
            )
    }

    fn render_tabs_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Tabs"),
            )
            .child(self.tabs_example.clone())
    }

    fn render_dropdown_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
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
            )
    }

    fn render_select_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Select"),
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
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Country"),
                            )
                            .child(self.country_select.clone()),
                    ),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_start()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .text_color(theme.fg_muted())
                            .child("Selected country:")
                            .child(
                                div()
                                    .text_color(theme.accent())
                                    .font_weight(FontWeight::BOLD)
                                    .child(
                                        self.country_select
                                            .read(cx)
                                            .selected
                                            .as_ref()
                                            .map(|c| format!("{:?}", c))
                                            .unwrap_or_else(|| "None".to_string()),
                                    ),
                            ),
                    ),
            )
    }

    fn render_field_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Field"),
            )
            .child(
                v_stack()
                    .gap_4()
                    .child(
                        field()
                            .label("Username")
                            .description("Enter your preferred username")
                            .required(true)
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.border())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("(input placeholder)"),
                            ),
                    )
                    .child(
                        field()
                            .label("Email")
                            .error("Please enter a valid email address")
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.danger())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("invalid@"),
                            ),
                    )
                    .child(
                        field()
                            .label("Department")
                            .label_position(LabelPosition::Beside)
                            .description("Select your department")
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(theme.border())
                                    .rounded(gpui::px(4.0))
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("(horizontal layout)"),
                            ),
                    ),
            )
    }

    fn render_input_group_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("InputGroup"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("With icon:"),
                            )
                            .child(
                                input_group(&self.input_with_icon, cx)
                                    .left_addon(InputAddon::icon(
                                        DefaultIcons::magnifying_glass(),
                                    )),
                            ),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("With text:"),
                            )
                            .child(
                                input_group(&self.input_with_text, cx)
                                    .left_addon(InputAddon::text("https://")),
                            ),
                    )
                    .child(
                        h_stack()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child("With button:"),
                            )
                            .child(
                                input_group(&self.input_with_button, cx)
                                    .right_addon(InputAddon::button(
                                        button("go-btn", "Go"),
                                    )),
                            ),
                    ),
            )
    }

    fn render_textarea_page(&self, cx: &Context<Self>) -> impl IntoElement {
        card()
            .title("Textarea")
            .description("Multi-line text input for longer content")
            .body(
                v_stack()
                    .gap_4()
                    .child(
                        field()
                            .label("Message")
                            .description("Tell us what's on your mind")
                            .child(
                                textarea(&self.textarea_example, cx)
                                    .placeholder("Type your message here...")
                                    .rows(4),
                            ),
                    )
                    .child(
                        field()
                            .label("Disabled")
                            .child(
                                textarea(&self.textarea_example, cx)
                                    .placeholder("This is disabled...")
                                    .rows(2)
                                    .disabled(true),
                            ),
                    ),
            )
    }

    fn render_avatar_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
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
            )
    }

    fn render_badge_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Badge"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(badge("Default"))
                    .child(badge("Secondary").secondary())
                    .child(badge("Outline").outline())
                    .child(badge("Destructive").destructive()),
            )
    }

    fn render_kbd_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Kbd"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .child(kbd("Esc"))
                    .child(kbd("Enter"))
                    .child(kbd("Tab"))
                    .child(kbd_combo(&["Ctrl", "C"]))
                    .child(kbd_combo(&["Cmd", "Shift", "P"])),
            )
            .child(
                h_stack()
                    .gap_2()
                    .items_center()
                    .mt_2()
                    .child(kbd("S").size(KbdSize::Small))
                    .child(kbd("M"))
                    .child(kbd("L").size(KbdSize::Large))
                    .child(
                        div()
                            .text_color(theme.fg_muted())
                            .child("(small / default / large)"),
                    ),
            )
    }

    fn render_label_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Label"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(label("Basic Label"))
                    .child(label("Required Field").required(true))
                    .child(label("Disabled Label").disabled(true)),
            )
    }

    fn render_loading_indicator_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("LoadingIndicator"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(loading_indicator().dots())
                    .child(loading_indicator().ellipsis())
                    .child(loading_indicator().dash())
                    .child(loading_indicator().star())
                    .child(loading_indicator().triangle())
                    .child(loading_indicator().braille())
                    .child(loading_indicator().braille_extended()),
            )
    }

    fn render_progress_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Progress"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(progress(0.25))
                    .child(progress(0.5))
                    .child(progress(0.75))
                    .child(progress(1.0).variant(ProgressVariant::Danger)),
            )
    }

    fn render_skeleton_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Skeleton"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .items_center()
                    .child(skeleton_avatar())
                    .child(
                        v_stack()
                            .gap_2()
                            .child(skeleton_text().w(px(150.0)))
                            .child(skeleton_text().w(px(100.0))),
                    ),
            )
            .child(
                h_stack()
                    .gap_4()
                    .mt_2()
                    .child(skeleton().w(px(80.0)).h(px(32.0)))
                    .child(skeleton().w(px(120.0)).h(px(32.0)))
                    .child(skeleton().w(px(60.0)).h(px(32.0)).circle()),
            )
            .child(div().mt_2().child(skeleton_card()))
    }

    fn render_alert_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Alert"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(alert("This is a default alert message."))
                    .child(
                        alert("Informational: Your session will expire in 5 minutes.").info(),
                    )
                    .child(alert("Success! Your changes have been saved.").success())
                    .child(alert("Warning: This action cannot be undone.").warning())
                    .child(alert("Error: Failed to connect to server.").destructive())
                    .child(
                        alert("New feature available!")
                            .info()
                            .title("Heads up!")
                            .id("dismissible-alert")
                            .dismissible(true),
                    ),
            )
    }

    fn render_tooltip_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Tooltip"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(
                        button("tooltip-btn-1", "Hover me")
                            .tooltip(tooltip("This is a tooltip")),
                    )
                    .child(
                        icon_button("tooltip-icon", DefaultIcons::info_circled())
                            .tooltip(tooltip("More information")),
                    )
                    .child(button("tooltip-btn-2", "Another one").tooltip(
                        tooltip("Tooltips work on any element with an id"),
                    )),
            )
    }

    fn render_card_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Card"),
            )
            .child(
                card()
                    .title("Card Title")
                    .description("A short description of the card content.")
                    .footer(
                        h_stack()
                            .gap_2()
                            .child(button("card-save", "Save"))
                            .child(button("card-cancel", "Cancel").disabled(true)),
                    ),
            )
    }

    fn render_aspect_ratio_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("AspectRatio"),
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
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("1:1 Square"),
                            )
                            .child(
                                aspect_ratio_square()
                                    .width(px(80.0))
                                    .child(
                                        div()
                                            .size_full()
                                            .bg(theme.accent())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_xs()
                                            .text_color(theme.bg())
                                            .child("1:1"),
                                    ),
                            ),
                    )
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("16:9 Video"),
                            )
                            .child(
                                aspect_ratio_video()
                                    .width(px(160.0))
                                    .child(
                                        div()
                                            .size_full()
                                            .bg(theme.surface_secondary())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_xs()
                                            .text_color(theme.fg())
                                            .child("16:9"),
                                    ),
                            ),
                    )
                    .child(
                        v_stack()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("4:3 Photo"),
                            )
                            .child(
                                aspect_ratio(4.0 / 3.0)
                                    .width(px(120.0))
                                    .child(
                                        div()
                                            .size_full()
                                            .bg(theme.accent_bg())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_xs()
                                            .text_color(theme.accent())
                                            .child("4:3"),
                                    ),
                            ),
                    ),
            )
    }

    fn render_breadcrumb_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Breadcrumb"),
            )
            .child(
                v_stack()
                    .gap_3()
                    .child(
                        breadcrumb("breadcrumb-1")
                            .item(breadcrumb_item("Home"))
                            .item(breadcrumb_item("Documents"))
                            .item(breadcrumb_item("Projects")),
                    )
                    .child(
                        breadcrumb("breadcrumb-2")
                            .separator(BreadcrumbSeparator::Chevron)
                            .item(breadcrumb_item("Settings"))
                            .item(breadcrumb_item("Account"))
                            .item(breadcrumb_item("Profile")),
                    )
                    .child(
                        breadcrumb("breadcrumb-3")
                            .separator(BreadcrumbSeparator::Arrow)
                            .item(breadcrumb_item("Level 1"))
                            .item(breadcrumb_item("Level 2"))
                            .item(breadcrumb_item("Current")),
                    ),
            )
    }

    fn render_separator_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Separator"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(div().text_sm().child("Content above"))
                    .child(separator())
                    .child(div().text_sm().child("Content below")),
            )
    }

    fn render_collapsible_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Collapsible"),
            )
            .child(
                v_stack()
                    .gap_2()
                    .child(self.collapsible_basic.clone())
                    .child(self.collapsible_nested.clone()),
            )
    }

    fn render_accordion_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Accordion"),
            )
            .child(self.accordion.clone())
    }

    fn render_scroll_area_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("ScrollArea"),
            )
            .child(
                h_stack()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Vertical scroll:"),
                            )
                            .child(
                                scroll_area("vertical-scroll-demo")
                                    .max_h(px(120.))
                                    .vertical()
                                    .child(
                                        v_stack()
                                            .gap_2()
                                            .p_2()
                                            .bg(theme.surface())
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded_sm()
                                            .children((1..=15).map(|i| {
                                                div().text_xs().child(format!("Item {}", i))
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child("Horizontal scroll:"),
                            )
                            .child(
                                scroll_area("horizontal-scroll-demo")
                                    .max_w(px(150.))
                                    .horizontal()
                                    .child(
                                        h_stack()
                                            .gap_2()
                                            .p_2()
                                            .bg(theme.surface())
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded_sm()
                                            .children((1..=10).map(|i| {
                                                div()
                                                    .px_3()
                                                    .py_1()
                                                    .bg(theme.accent_bg())
                                                    .rounded_sm()
                                                    .text_xs()
                                                    .child(format!("Tag {}", i))
                                            })),
                                    ),
                            ),
                    ),
            )
    }

    fn render_list_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("List"),
            )
            .child(
                div()
                    .h(px(250.))
                    .border_1()
                    .border_color(theme.border())
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        List::new(
                            "showcase-list",
                            vec![
                                ListEntry::header("Conflicts"),
                                ListEntry::item("f-1", |_w, _cx| {
                                    div().px_2().child("src/services.rs").into_any_element()
                                }),
                                ListEntry::header("Changes"),
                                ListEntry::item("f-2", |_w, _cx| {
                                    div().px_2().child("src/main.rs").into_any_element()
                                }),
                                ListEntry::item("f-3", |_w, _cx| {
                                    div()
                                        .px_2()
                                        .child("src/services/auth.rs")
                                        .into_any_element()
                                }),
                                ListEntry::item("f-4", |_w, _cx| {
                                    div().px_2().child("src/ui/auth.rs").into_any_element()
                                }),
                                ListEntry::item("f-5", |_w, _cx| {
                                    div()
                                        .px_2()
                                        .child("src/utils/helpers.rs")
                                        .into_any_element()
                                }),
                                ListEntry::header("New"),
                                ListEntry::item("f-6", |_w, _cx| {
                                    div().px_2().child("build.rs").into_any_element()
                                }),
                                ListEntry::item("f-7", |_w, _cx| {
                                    div().px_2().child("Cargo.toml").into_any_element()
                                }),
                                ListEntry::item("f-8", |_w, _cx| {
                                    div().px_2().child("src/lib.rs").into_any_element()
                                }),
                            ],
                        )
                        .render(window, cx),
                    ),
            )
    }

    fn render_popover_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Popover"),
            )
            .child(self.popover_example.clone())
    }

    fn render_dialog_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Dialog"),
            )
            .child(
                button("open-dialog", "Open Dialog")
                    .on_click(cx.listener(|showcase, _, window, cx| {
                        showcase.dialog_example.update(cx, |dialog, cx| {
                            dialog.open(window, cx);
                        });
                    })),
            )
    }

    fn render_context_menu_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Context Menu"),
            )
            .child(self.context_menu_example.clone())
    }

    fn render_toast_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_stack()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.fg_muted())
                    .child("Toast"),
            )
            .child(
                h_stack()
                    .gap_2()
                    .child(
                        button("toast-default", "Default")
                            .on_click(cx.listener(|_, _, window, cx| {
                                cx.toast("This is a default toast").show(window, cx);
                            })),
                    )
                    .child(
                        button("toast-success", "Success")
                            .on_click(cx.listener(|_, _, window, cx| {
                                cx.toast("Changes saved successfully")
                                    .success()
                                    .show(window, cx);
                            })),
                    )
                    .child(
                        button("toast-warning", "Warning")
                            .on_click(cx.listener(|_, _, window, cx| {
                                cx.toast("Please check your input")
                                    .warning()
                                    .show(window, cx);
                            })),
                    ),
            )
    }

    fn render_markdown_page(&self, _cx: &Context<Self>) -> impl IntoElement {
        MarkdownElement::new(self.markdown.clone())
    }
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_page: SharedString = self.active_page.borrow().clone();

        // Capture owned theme colors for sidebar before any mutable borrows
        let bg = cx.theme().bg();
        let fg = cx.theme().fg();
        let border = cx.theme().border();
        let surface = cx.theme().surface();

        // Build nav entries for the sidebar
        let nav_pages: Vec<(&str, Vec<&str>)> = vec![
            ("Actions", vec!["Button", "ButtonGroup", "Icon Button"]),
            (
                "Inputs",
                vec![
                    "Checkbox",
                    "Switch",
                    "RadioGroup",
                    "ToggleGroup",
                    "Tabs",
                    "Dropdown",
                    "Select",
                    "Field",
                    "InputGroup",
                    "Textarea",
                ],
            ),
            (
                "Display",
                vec![
                    "Avatar",
                    "Badge",
                    "Kbd",
                    "Label",
                    "LoadingIndicator",
                    "Progress",
                    "Skeleton",
                    "Alert",
                    "Tooltip",
                    "Card",
                    "AspectRatio",
                ],
            ),
            (
                "Layout",
                vec![
                    "Breadcrumb",
                    "Separator",
                    "Collapsible",
                    "Accordion",
                    "ScrollArea",
                    "List",
                ],
            ),
            (
                "Overlay",
                vec!["Popover", "Dialog", "Context Menu", "Toast"],
            ),
            ("Content", vec!["Markdown"]),
        ];

        let mut entries: Vec<ListEntry> = Vec::new();
        for (category, pages) in &nav_pages {
            entries.push(ListEntry::header(*category));
            for page_name in pages {
                let page_cell = self.active_page.clone();
                let name: SharedString = SharedString::from(*page_name);
                let name_for_render = name.clone();
                let name_for_click = name.clone();
                let is_selected = current_page == name;
                entries.push(
                    ListEntry::item(
                        SharedString::from(format!("nav-{}", page_name)),
                        move |_w, _cx| {
                            div()
                                .px_2()
                                .child(name_for_render.clone())
                                .into_any_element()
                        },
                    )
                    .on_click({
                        let cell = page_cell.clone();
                        move |_, window, _cx| {
                            *cell.borrow_mut() = name_for_click.clone();
                            window.refresh();
                        }
                    })
                    .selected(is_selected),
                );
            }
        }

        let sidebar = div()
            .w(px(200.))
            .min_h_full()
            .border_r_1()
            .border_color(border)
            .bg(surface)
            .flex_shrink_0()
            .child(List::new("nav-list", entries).render(window, cx));

        let content = match current_page.as_ref() {
            "Button" => self.render_button_page(window, cx).into_any_element(),
            "ButtonGroup" => self.render_button_group_page(cx).into_any_element(),
            "Icon Button" => self.render_icon_button_page(window, cx).into_any_element(),
            "Checkbox" => self.render_checkbox_page(cx).into_any_element(),
            "Switch" => self.render_switch_page(cx).into_any_element(),
            "RadioGroup" => self.render_radio_group_page(cx).into_any_element(),
            "ToggleGroup" => self.render_toggle_group_page(cx).into_any_element(),
            "Tabs" => self.render_tabs_page(cx).into_any_element(),
            "Dropdown" => self.render_dropdown_page(cx).into_any_element(),
            "Select" => self.render_select_page(cx).into_any_element(),
            "Field" => self.render_field_page(cx).into_any_element(),
            "InputGroup" => self.render_input_group_page(cx).into_any_element(),
            "Textarea" => self.render_textarea_page(cx).into_any_element(),
            "Avatar" => self.render_avatar_page(cx).into_any_element(),
            "Badge" => self.render_badge_page(cx).into_any_element(),
            "Kbd" => self.render_kbd_page(cx).into_any_element(),
            "Label" => self.render_label_page(cx).into_any_element(),
            "LoadingIndicator" => self.render_loading_indicator_page(cx).into_any_element(),
            "Progress" => self.render_progress_page(cx).into_any_element(),
            "Skeleton" => self.render_skeleton_page(cx).into_any_element(),
            "Alert" => self.render_alert_page(cx).into_any_element(),
            "Tooltip" => self.render_tooltip_page(cx).into_any_element(),
            "Card" => self.render_card_page(cx).into_any_element(),
            "AspectRatio" => self.render_aspect_ratio_page(cx).into_any_element(),
            "Breadcrumb" => self.render_breadcrumb_page(cx).into_any_element(),
            "Separator" => self.render_separator_page(cx).into_any_element(),
            "Collapsible" => self.render_collapsible_page(cx).into_any_element(),
            "Accordion" => self.render_accordion_page(cx).into_any_element(),
            "ScrollArea" => self.render_scroll_area_page(cx).into_any_element(),
            "List" => self.render_list_page(window, cx).into_any_element(),
            "Popover" => self.render_popover_page(cx).into_any_element(),
            "Dialog" => self.render_dialog_page(window, cx).into_any_element(),
            "Context Menu" => self.render_context_menu_page(cx).into_any_element(),
            "Toast" => self.render_toast_page(window, cx).into_any_element(),
            "Markdown" => self.render_markdown_page(cx).into_any_element(),
            _ => div().child("Unknown page").into_any_element(),
        };

        h_stack()
            .bg(bg)
            .text_color(fg)
            .size_full()
            .overflow_hidden()
            .child(sidebar)
            .child(
                div()
                    .id("content-area")
                    .flex_1()
                    .overflow_y_scroll()
                    .min_h_full()
                    .p_8()
                    .child(content),
            )
            .child(self.dialog_example.clone())
            .child(cx.toast_manager().clone())
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            cx.set_menus(vec![Menu {
                name: "GPUIKit Showcase".into(),
                items: vec![],
                disabled: false,
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
                    window.focus(&showcase.focus_handle, cx);
                    cx.activate(true);
                })
                .unwrap();
        });
}
