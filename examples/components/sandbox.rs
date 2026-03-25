//! Components Sandbox - An interactive example for testing UI components.
//!
//! Use the sidebar list to navigate between component categories.

#![allow(missing_docs)]

use gpui::{
    actions, div, linear_color_stop, linear_gradient, prelude::*, px, rgb, size, App, Application,
    Background, Bounds, Context, Div, Entity, FocusHandle, Focusable, FontWeight, Hsla, KeyBinding,
    ParentElement, Render, SharedString, Stateful, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_platform;
use gpuikit::elements::button::button;
use gpuikit::elements::checkbox::{checkbox, Checkbox, CheckboxChanged};
use gpuikit::elements::dropdown::{dropdown, DropdownState};
use gpuikit::elements::list::{List, ListEntry};
use gpuikit::elements::loading_indicator::loading_indicator;
use gpuikit::elements::progress::{progress, ProgressVariant};
use gpuikit::elements::radio_group::{radio_group, radio_option, RadioGroup, RadioGroupChanged};
use gpuikit::elements::slider::{Slider, SliderChanged};
use gpuikit::elements::toggle::{toggle, Toggle, ToggleChanged};
use gpuikit::layout::{h_stack, v_stack};

actions!(components_sandbox, [ResetAll]);

enum Theme {
    Background,
    Foreground,
    ForegroundMuted,
    AccentBlue,
    Surface,
}

impl Theme {
    fn hsla(&self) -> Hsla {
        match self {
            Theme::Background => rgb(0x1A1D24).into(),
            Theme::Foreground => rgb(0xE8E4DC).into(),
            Theme::ForegroundMuted => Self::Foreground.hsla().alpha(0.6),
            Theme::AccentBlue => rgb(0x4A9EFF).into(),
            Theme::Surface => rgb(0x252830).into(),
        }
    }
}

enum ThemeGradient {
    BackgroundToSurface,
}

impl ThemeGradient {
    fn gradient(&self) -> Background {
        match self {
            Self::BackgroundToSurface => linear_gradient(
                180.0,
                linear_color_stop(Theme::Background.hsla(), 0.0),
                linear_color_stop(Theme::Surface.hsla(), 1.0),
            ),
        }
    }
}

fn sidebar_section(title: impl Into<Option<&'static str>>) -> Div {
    let title = title.into();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .border_b_1()
        .border_color(Theme::ForegroundMuted.hsla().alpha(0.2))
        .when_some(title, |this, title| {
            this.child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(Theme::ForegroundMuted.hsla())
                    .child(title),
            )
        })
}

fn stat_row(label: &'static str, value: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .justify_between()
        .text_xs()
        .child(div().text_color(Theme::ForegroundMuted.hsla()).child(label))
        .child(
            div()
                .text_color(Theme::Foreground.hsla())
                .child(value.into()),
        )
}

fn component_section(title: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .bg(Theme::Surface.hsla().alpha(0.5))
        .rounded_lg()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(Theme::AccentBlue.hsla())
                .child(title),
        )
}

fn toggle_button(id: &'static str, label: &'static str, active: bool) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .text_xs()
        .rounded_sm()
        .cursor_pointer()
        .when(active, |this| {
            this.bg(Theme::AccentBlue.hsla())
                .text_color(Theme::Foreground.hsla())
        })
        .when(!active, |this| {
            this.bg(Theme::ForegroundMuted.hsla().alpha(0.2))
                .text_color(Theme::ForegroundMuted.hsla())
                .hover(|style| style.bg(Theme::ForegroundMuted.hsla().alpha(0.3)))
        })
        .child(label)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum LoadingStyle {
    Dots,
    Ellipsis,
    Dash,
    Star,
    Triangle,
    Braille,
}

impl LoadingStyle {
    const ALL: [LoadingStyle; 6] = [
        Self::Dots,
        Self::Ellipsis,
        Self::Dash,
        Self::Star,
        Self::Triangle,
        Self::Braille,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Dots => "Dots",
            Self::Ellipsis => "Ellipsis",
            Self::Dash => "Dash",
            Self::Star => "Star",
            Self::Triangle => "Triangle",
            Self::Braille => "Braille",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NotificationSetting {
    All,
    Important,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Category {
    Controls,
    Inputs,
    Feedback,
    List,
}

impl Category {
    fn label(&self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Inputs => "Inputs",
            Self::Feedback => "Feedback",
            Self::List => "List",
        }
    }
}

struct ComponentsSandbox {
    focus_handle: FocusHandle,
    active_category: Category,

    // Controls
    toggle_enabled: Entity<Toggle>,
    toggle_disabled: Entity<Toggle>,
    toggle_with_label: Entity<Toggle>,
    checkbox_unchecked: Entity<Checkbox>,
    checkbox_checked: Entity<Checkbox>,
    checkbox_disabled: Entity<Checkbox>,
    radio_group: Entity<RadioGroup<NotificationSetting>>,

    // Inputs
    slider_value: Entity<Slider>,
    slider_range: Entity<Slider>,
    slider_stepped: Entity<Slider>,

    // Feedback
    progress_value: f32,
    progress_slider: Entity<Slider>,
    loading_dropdown: Entity<DropdownState<LoadingStyle>>,

    // Global disabled state
    all_disabled: bool,

    // Stats
    toggle_count: usize,
    slider_changes: usize,
    checkbox_changes: usize,
    button_clicks: usize,

    _subscriptions: Vec<gpui::Subscription>,
}

impl ComponentsSandbox {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let toggle_enabled = cx.new(|_cx| toggle("toggle-enabled", true));
        let toggle_disabled = cx.new(|_cx| toggle("toggle-disabled", false).disabled(true));
        let toggle_with_label =
            cx.new(|_cx| toggle("toggle-labeled", false).label("Notifications"));

        let slider_value = cx.new(|_cx| {
            Slider::new("slider-value", 50.0, 0.0..=100.0)
                .label("Value")
                .step(1.0)
        });
        let slider_range = cx.new(|_cx| {
            Slider::new("slider-range", 1.5, 0.0..=3.0)
                .label("Multiplier")
                .step(0.1)
        });
        let slider_stepped = cx.new(|_cx| {
            Slider::new("slider-stepped", 5.0, 0.0..=10.0)
                .label("Steps")
                .step(1.0)
        });

        let checkbox_unchecked = cx.new(|_cx| checkbox("cb-unchecked", false).label("Unchecked"));
        let checkbox_checked = cx.new(|_cx| checkbox("cb-checked", true).label("Checked"));
        let checkbox_disabled = cx.new(|_cx| {
            checkbox("cb-disabled", true)
                .label("Disabled")
                .disabled(true)
        });

        let radio_group = cx.new(|_cx| {
            radio_group(
                "theme-radio",
                vec![
                    radio_option(NotificationSetting::All, "All notifications"),
                    radio_option(NotificationSetting::Important, "Important only"),
                    radio_option(NotificationSetting::None, "None"),
                ],
            )
            .selected(NotificationSetting::Important)
        });

        let progress_slider = cx.new(|_cx| {
            Slider::new("progress-control", 0.5, 0.0..=1.0)
                .label("Progress")
                .step(0.05)
        });

        let loading_options: Vec<(LoadingStyle, &'static str)> =
            LoadingStyle::ALL.iter().map(|s| (*s, s.label())).collect();
        let loading_dropdown = cx.new(|_cx| {
            DropdownState::new(
                dropdown("loading-style", loading_options, LoadingStyle::Dots).full_width(true),
            )
        });

        // Subscribe to state changes
        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe_in(
            &toggle_enabled,
            _window,
            |this, _, _: &ToggleChanged, _, cx| {
                this.toggle_count += 1;
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &toggle_with_label,
            _window,
            |this, _, _: &ToggleChanged, _, cx| {
                this.toggle_count += 1;
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe_in(
            &slider_value,
            _window,
            |this, _, _: &SliderChanged, _, cx| {
                this.slider_changes += 1;
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &slider_range,
            _window,
            |this, _, _: &SliderChanged, _, cx| {
                this.slider_changes += 1;
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &slider_stepped,
            _window,
            |this, _, _: &SliderChanged, _, cx| {
                this.slider_changes += 1;
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe_in(
            &checkbox_unchecked,
            _window,
            |this, _, _: &CheckboxChanged, _, cx| {
                this.checkbox_changes += 1;
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &checkbox_checked,
            _window,
            |this, _, _: &CheckboxChanged, _, cx| {
                this.checkbox_changes += 1;
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe_in(
            &radio_group,
            _window,
            |this, _, _: &RadioGroupChanged<NotificationSetting>, _, cx| {
                cx.notify();
                let _ = this;
            },
        ));

        subscriptions.push(cx.subscribe_in(
            &progress_slider,
            _window,
            |this, entity, _: &SliderChanged, _, cx| {
                this.progress_value = entity.read(cx).value();
                this.slider_changes += 1;
                cx.notify();
            },
        ));

        Self {
            focus_handle: cx.focus_handle(),
            active_category: Category::Controls,
            toggle_enabled,
            toggle_disabled,
            toggle_with_label,
            checkbox_unchecked,
            checkbox_checked,
            checkbox_disabled,
            radio_group,
            slider_value,
            slider_range,
            slider_stepped,
            progress_value: 0.5,
            progress_slider,
            loading_dropdown,
            all_disabled: false,
            toggle_count: 0,
            slider_changes: 0,
            checkbox_changes: 0,
            button_clicks: 0,
            _subscriptions: subscriptions,
        }
    }

    fn render_controls(&self, cx: &Context<Self>) -> impl IntoElement {
        let radio_selected = self.radio_group.read(cx).get_selected().cloned();

        v_stack()
            .gap_4()
            .child(
                component_section("Toggle").child(
                    h_stack()
                        .gap_6()
                        .items_center()
                        .child(
                            v_stack()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(Theme::ForegroundMuted.hsla())
                                        .child("Enabled"),
                                )
                                .child(self.toggle_enabled.clone()),
                        )
                        .child(
                            v_stack()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(Theme::ForegroundMuted.hsla())
                                        .child("Disabled"),
                                )
                                .child(self.toggle_disabled.clone()),
                        )
                        .child(
                            v_stack()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(Theme::ForegroundMuted.hsla())
                                        .child("With Label"),
                                )
                                .child(self.toggle_with_label.clone()),
                        ),
                ),
            )
            .child(
                component_section("Checkbox").child(
                    h_stack()
                        .gap_6()
                        .child(self.checkbox_unchecked.clone())
                        .child(self.checkbox_checked.clone())
                        .child(self.checkbox_disabled.clone()),
                ),
            )
            .child(
                component_section("Radio Group")
                    .child(self.radio_group.clone())
                    .child(
                        div()
                            .text_xs()
                            .text_color(Theme::ForegroundMuted.hsla())
                            .mt_2()
                            .child(format!("Selected: {:?}", radio_selected)),
                    ),
            )
            .child(
                component_section("Button").child(
                    h_stack()
                        .gap_2()
                        .child(button("btn-primary", "Primary").on_click(cx.listener(
                            |this, _event, _window, cx| {
                                this.button_clicks += 1;
                                cx.notify();
                            },
                        )))
                        .child(button("btn-secondary", "Secondary").on_click(cx.listener(
                            |this, _event, _window, cx| {
                                this.button_clicks += 1;
                                cx.notify();
                            },
                        )))
                        .child(button("btn-disabled", "Disabled").disabled(true)),
                ),
            )
    }

    fn render_inputs(&self, cx: &Context<Self>) -> impl IntoElement {
        let slider_val = self.slider_value.read(cx).value();
        let slider_range = self.slider_range.read(cx).value();
        let slider_stepped = self.slider_stepped.read(cx).value();

        v_stack().gap_4().child(
            component_section("Slider")
                .child(
                    v_stack()
                        .gap_4()
                        .child(self.slider_value.clone())
                        .child(self.slider_range.clone())
                        .child(self.slider_stepped.clone()),
                )
                .child(
                    div()
                        .mt_2()
                        .text_xs()
                        .text_color(Theme::ForegroundMuted.hsla())
                        .child(format!(
                            "Values: {:.0}, {:.1}x, {:.0}",
                            slider_val, slider_range, slider_stepped
                        )),
                ),
        )
    }

    fn render_feedback(&self, cx: &Context<Self>) -> impl IntoElement {
        let loading_style = self.loading_dropdown.read(cx).selected;

        v_stack()
            .gap_4()
            .child(
                component_section("Progress")
                    .child(self.progress_slider.clone())
                    .child(
                        v_stack()
                            .gap_2()
                            .mt_2()
                            .child(progress(self.progress_value))
                            .child(progress(self.progress_value).variant(ProgressVariant::Success))
                            .child(progress(self.progress_value).variant(ProgressVariant::Danger)),
                    ),
            )
            .child(
                component_section("Loading Indicator").child(
                    h_stack()
                        .gap_4()
                        .items_center()
                        .child(match loading_style {
                            LoadingStyle::Dots => loading_indicator().dots().into_any_element(),
                            LoadingStyle::Ellipsis => {
                                loading_indicator().ellipsis().into_any_element()
                            }
                            LoadingStyle::Dash => loading_indicator().dash().into_any_element(),
                            LoadingStyle::Star => loading_indicator().star().into_any_element(),
                            LoadingStyle::Triangle => {
                                loading_indicator().triangle().into_any_element()
                            }
                            LoadingStyle::Braille => {
                                loading_indicator().braille().into_any_element()
                            }
                        })
                        .child(
                            div()
                                .text_sm()
                                .text_color(Theme::ForegroundMuted.hsla())
                                .child("Loading..."),
                        ),
                ),
            )
    }

    fn render_list(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_stack().gap_4().child(
            component_section("List").child(
                div()
                    .h(px(300.))
                    .border_1()
                    .border_color(Theme::ForegroundMuted.hsla().alpha(0.2))
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        List::new(
                            "sandbox-list",
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
                                    div().px_2().child("src/services/auth.rs").into_any_element()
                                }),
                                ListEntry::item("f-4", |_w, _cx| {
                                    div().px_2().child("src/ui/auth.rs").into_any_element()
                                }),
                                ListEntry::item("f-5", |_w, _cx| {
                                    div().px_2().child("src/utils/helpers.rs").into_any_element()
                                }),
                                ListEntry::item("f-6", |_w, _cx| {
                                    div().px_2().child("src/models/user.rs").into_any_element()
                                }),
                                ListEntry::item("f-7", |_w, _cx| {
                                    div().px_2().child("README.md").into_any_element()
                                }),
                                ListEntry::header("New"),
                                ListEntry::item("f-8", |_w, _cx| {
                                    div().px_2().child("build.rs").into_any_element()
                                }),
                                ListEntry::item("f-9", |_w, _cx| {
                                    div().px_2().child("Cargo.toml").into_any_element()
                                }),
                                ListEntry::item("f-10", |_w, _cx| {
                                    div().px_2().child("src/lib.rs").into_any_element()
                                }),
                                ListEntry::item("f-11", |_w, _cx| {
                                    div().px_2().child("tests/auth.rs").into_any_element()
                                }),
                            ],
                        )
                        .render(window, cx),
                    ),
            ),
        )
    }

    fn reset_all(&mut self, _: &ResetAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_count = 0;
        self.slider_changes = 0;
        self.checkbox_changes = 0;
        self.button_clicks = 0;
        self.progress_value = 0.5;

        self.slider_value.update(cx, |s, cx| s.set_value(50.0, cx));
        self.slider_range.update(cx, |s, cx| s.set_value(1.5, cx));
        self.slider_stepped
            .update(cx, |s, cx| s.set_value(5.0, cx));
        self.progress_slider
            .update(cx, |s, cx| s.set_value(0.5, cx));

        self.toggle_enabled
            .update(cx, |t, cx| t.set_enabled(true, cx));
        self.toggle_with_label
            .update(cx, |t, cx| t.set_enabled(false, cx));

        cx.notify();
    }
}

impl Focusable for ComponentsSandbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ComponentsSandbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active_category;
        let slider_val = self.slider_value.read(cx).value();
        let slider_range = self.slider_range.read(cx).value();
        let slider_stepped = self.slider_stepped.read(cx).value();

        div()
            .id("components-sandbox")
            .key_context("ComponentsSandbox")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::reset_all))
            .flex()
            .flex_row()
            .bg(ThemeGradient::BackgroundToSurface.gradient())
            .text_color(Theme::Foreground.hsla())
            .size_full()
            // Left sidebar - Category nav
            .child(
                div()
                    .w(px(180.))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .bg(Theme::Background.hsla().alpha(0.5))
                    .border_r_1()
                    .border_color(Theme::ForegroundMuted.hsla().alpha(0.2))
                    .child(
                        div()
                            .p_3()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(Theme::AccentBlue.hsla())
                            .child("Components"),
                    )
                    .child(
                        v_stack().px_2().gap_1().children(
                            [
                                Category::Controls,
                                Category::Inputs,
                                Category::Feedback,
                                Category::List,
                            ]
                            .into_iter()
                            .map(|cat| {
                                let is_active = cat == active;
                                div()
                                    .id(SharedString::from(cat.label()))
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .when(is_active, |el| {
                                        el.bg(Theme::AccentBlue.hsla().alpha(0.15))
                                            .text_color(Theme::AccentBlue.hsla())
                                    })
                                    .when(!is_active, |el| {
                                        el.text_color(Theme::ForegroundMuted.hsla())
                                            .hover(|s| {
                                                s.bg(Theme::ForegroundMuted.hsla().alpha(0.1))
                                            })
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.active_category = cat;
                                        cx.notify();
                                    }))
                                    .child(cat.label())
                            }),
                        ),
                    ),
            )
            // Center - Active category content
            .child(
                div()
                    .id("components-panel")
                    .flex_1()
                    .flex()
                    .flex_col()
                    .p_6()
                    .overflow_y_scroll()
                    .child(match active {
                        Category::Controls => self.render_controls(cx).into_any_element(),
                        Category::Inputs => self.render_inputs(cx).into_any_element(),
                        Category::Feedback => self.render_feedback(cx).into_any_element(),
                        Category::List => self.render_list(window, cx).into_any_element(),
                    }),
            )
            // Right sidebar - Stats & controls
            .child(
                div()
                    .id("sidebar")
                    .w(px(240.))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .bg(Theme::Background.hsla().alpha(0.5))
                    .border_l_1()
                    .border_color(Theme::ForegroundMuted.hsla().alpha(0.2))
                    // Disabled toggle
                    .child(
                        sidebar_section(None).child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    toggle_button("disabled-off", "Interactive", !self.all_disabled)
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.all_disabled = false;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    toggle_button("disabled-on", "Disabled", self.all_disabled)
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.all_disabled = true;
                                            cx.notify();
                                        })),
                                ),
                        ),
                    )
                    // Loading style selector
                    .child(sidebar_section("Loading Style").child(self.loading_dropdown.clone()))
                    // Slider values
                    .child(
                        sidebar_section("Slider Values")
                            .child(stat_row("Value", format!("{:.0}", slider_val)))
                            .child(stat_row("Multiplier", format!("{:.1}x", slider_range)))
                            .child(stat_row("Steps", format!("{:.0}", slider_stepped))),
                    )
                    // Stats
                    .child(
                        sidebar_section("Interaction Stats")
                            .child(stat_row(
                                "Toggle changes",
                                format!("{}", self.toggle_count),
                            ))
                            .child(stat_row(
                                "Slider changes",
                                format!("{}", self.slider_changes),
                            ))
                            .child(stat_row(
                                "Checkbox changes",
                                format!("{}", self.checkbox_changes),
                            ))
                            .child(stat_row(
                                "Button clicks",
                                format!("{}", self.button_clicks),
                            )),
                    )
                    // Progress
                    .child(
                        sidebar_section("Progress Value").child(stat_row(
                            "Current",
                            format!("{:.0}%", self.progress_value * 100.0),
                        )),
                    )
                    // Reset
                    .child(
                        sidebar_section(None).child(
                            button("reset-btn", "Reset All").on_click(cx.listener(
                                |this, _event, window, cx| {
                                    this.reset_all(&ResetAll, window, cx);
                                },
                            )),
                        ),
                    )
                    // Keybindings
                    .child(
                        sidebar_section("Keybindings").child(
                            div()
                                .flex()
                                .justify_between()
                                .gap_2()
                                .text_xs()
                                .child(
                                    div()
                                        .px_1()
                                        .bg(Theme::ForegroundMuted.hsla().alpha(0.2))
                                        .rounded_sm()
                                        .text_color(Theme::Foreground.hsla())
                                        .child("Ctrl+R"),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(Theme::ForegroundMuted.hsla())
                                        .child("Reset all"),
                                ),
                        ),
                    ),
            )
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);

            cx.bind_keys([KeyBinding::new("ctrl-r", ResetAll, None)]);

            let bounds = Bounds::centered(None, size(px(1100.), px(800.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| ComponentsSandbox::new(window, cx));
                    let focus_handle = view.read(cx).focus_handle.clone();
                    window.focus(&focus_handle, cx);
                    view
                },
            )
            .unwrap();

            cx.activate(true);
        });
}
