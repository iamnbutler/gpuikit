use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, Entity, FocusHandle, FontWeight,
    IntoElement, Menu, ParentElement, Render, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};
use gpuikit::{
    elements::{
        avatar::avatar,
        button::button,
        dropdown::{dropdown, DropdownState},
        icon_button::icon_button,
    },
    layout::{h_stack, v_stack},
    DefaultIcons,
};
use gpuikit_theme::{self, ActiveTheme, Themeable};

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

        Self {
            focus_handle: cx.focus_handle(),
            click_count: 0,
            toggled_count: 0,
            size_dropdown,
            priority_dropdown,
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
            .bg(theme.bg())
            .text_color(theme.fg())
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
                                    .text_color(theme.accent())
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
                            .child(icon_button("icon-search", DefaultIcons::magnifying_glass()))
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
                                    .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                        if *toggled {
                                            showcase.toggled_count += 1;
                                        } else {
                                            showcase.toggled_count =
                                                showcase.toggled_count.saturating_sub(1);
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                icon_button("toggle-heart", DefaultIcons::heart())
                                    .use_state()
                                    .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                        if *toggled {
                                            showcase.toggled_count += 1;
                                        } else {
                                            showcase.toggled_count =
                                                showcase.toggled_count.saturating_sub(1);
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                icon_button("toggle-bell", DefaultIcons::bell())
                                    .use_state()
                                    .on_toggle(cx.listener(|showcase, toggled, _window, cx| {
                                        if *toggled {
                                            showcase.toggled_count += 1;
                                        } else {
                                            showcase.toggled_count =
                                                showcase.toggled_count.saturating_sub(1);
                                        }
                                        cx.notify();
                                    })),
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
                                        div().text_sm().text_color(theme.fg_muted()).child("Size"),
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
                    .child(h_stack().gap_2().child(
                        avatar("https://avatars.githubusercontent.com/u/1714999?v=4").size(px(32.)),
                    )),
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
