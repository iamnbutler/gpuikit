//! Input Sandbox - A simple example for testing single-line and multi-line inputs.
//!
//! This example demonstrates gpuikit's input components including:
//! - Single-line text input
//! - Multi-line text area
//! - Dropdowns for font and sample text selection
//! - Sliders for font size and line height

#![allow(missing_docs)]

mod fixtures;

use gpui::{
    actions, div, linear_color_stop, linear_gradient, prelude::*, px, rgb, size, App, Application,
    Background, Bounds, Context, DefiniteLength, Div, Entity, FocusHandle, Focusable, FontWeight,
    Hsla, KeyBinding, SharedString, Stateful, Window, WindowBounds, WindowOptions,
};
use gpuikit::elements::dropdown::{dropdown, DropdownChanged, DropdownState};
use gpuikit::elements::input::{input, text_area};
use gpuikit::elements::slider::{Slider, SliderChanged};
use gpuikit::input::{bind_input_keys, InputState};

use fixtures::SampleText;

const SCROLL_GRADIENT_HEIGHT: f32 = 128.0;

actions!(input_sandbox, [ToggleMode]);

enum Theme {
    Background,
    Foreground,
    ForegroundMuted,
    AccentLightBlue,
    AccentLightPink,
    AccentDarkBlue,
    AccentDarkPink,
}

impl Theme {
    fn hsla(&self) -> Hsla {
        match self {
            Theme::Background => rgb(0x0E141D).into(),
            Theme::Foreground => rgb(0xF7F1E5).into(),
            Theme::ForegroundMuted => Self::Foreground.hsla().alpha(0.7),
            Theme::AccentLightBlue => rgb(0x2B64BE).into(),
            Theme::AccentLightPink => rgb(0xF2B2CB).into(),
            Theme::AccentDarkBlue => rgb(0x214EB1).into(),
            Theme::AccentDarkPink => rgb(0xE47BCC).into(),
        }
    }
}

enum ThemeGradient {
    DarkToBlue,
    TransparentToBlue,
    BlackToTransparent,
}

impl ThemeGradient {
    fn gradient(&self) -> Background {
        match self {
            Self::DarkToBlue => linear_gradient(
                180.0,
                linear_color_stop(Theme::Background.hsla(), 0.5),
                linear_color_stop(Theme::AccentDarkBlue.hsla(), 0.95),
            ),
            Self::TransparentToBlue => linear_gradient(
                180.0,
                linear_color_stop(Theme::AccentDarkBlue.hsla().opacity(0.), 0.0),
                linear_color_stop(Theme::AccentDarkBlue.hsla().opacity(1.), 0.8),
            ),
            Self::BlackToTransparent => linear_gradient(
                0.0,
                linear_color_stop(Theme::Background.hsla().opacity(1.), 0.8),
                linear_color_stop(Theme::Background.hsla().opacity(0.), 0.0),
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum FontWeightOption {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
}

impl FontWeightOption {
    const ALL: [FontWeightOption; 9] = [
        Self::Thin,
        Self::ExtraLight,
        Self::Light,
        Self::Normal,
        Self::Medium,
        Self::Semibold,
        Self::Bold,
        Self::ExtraBold,
        Self::Black,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Thin => "Thin (100)",
            Self::ExtraLight => "Extra Light (200)",
            Self::Light => "Light (300)",
            Self::Normal => "Normal (400)",
            Self::Medium => "Medium (500)",
            Self::Semibold => "Semibold (600)",
            Self::Bold => "Bold (700)",
            Self::ExtraBold => "Extra Bold (800)",
            Self::Black => "Black (900)",
        }
    }

    fn to_font_weight(&self) -> FontWeight {
        match self {
            Self::Thin => FontWeight::THIN,
            Self::ExtraLight => FontWeight::EXTRA_LIGHT,
            Self::Light => FontWeight::LIGHT,
            Self::Normal => FontWeight::NORMAL,
            Self::Medium => FontWeight::MEDIUM,
            Self::Semibold => FontWeight::SEMIBOLD,
            Self::Bold => FontWeight::BOLD,
            Self::ExtraBold => FontWeight::EXTRA_BOLD,
            Self::Black => FontWeight::BLACK,
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

fn toggle_button(id: &'static str, label: &'static str, active: bool) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .text_xs()
        .rounded_sm()
        .cursor_pointer()
        .when(active, |this| {
            this.bg(Theme::AccentLightBlue.hsla())
                .text_color(Theme::Foreground.hsla())
        })
        .when(!active, |this| {
            this.bg(Theme::ForegroundMuted.hsla().alpha(0.2))
                .text_color(Theme::ForegroundMuted.hsla())
                .hover(|style| style.bg(Theme::ForegroundMuted.hsla().alpha(0.3)))
        })
        .child(label)
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

fn key_row(key: &'static str, description: &'static str) -> Div {
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
                .child(key),
        )
        .child(
            div()
                .flex_1()
                .text_color(Theme::ForegroundMuted.hsla())
                .child(description),
        )
}

struct InputSandbox {
    multiline_input: Entity<InputState>,
    singleline_input: Entity<InputState>,
    use_multiline: bool,
    sample_dropdown: Entity<DropdownState<SampleText>>,
    font_dropdown: Entity<DropdownState<SharedString>>,
    weight_dropdown: Entity<DropdownState<FontWeightOption>>,
    font_size_slider: Entity<Slider>,
    line_height_slider: Entity<Slider>,
    selected_font: SharedString,
    selected_weight: FontWeight,
    _subscriptions: Vec<gpui::Subscription>,
}

impl InputSandbox {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial_sample = SampleText::Typography;

        let available_fonts: Vec<SharedString> = cx
            .text_system()
            .all_font_names()
            .into_iter()
            .take(999)
            .map(SharedString::from)
            .collect();

        let default_font: SharedString = available_fonts
            .iter()
            .find(|f| f.as_ref() == "Helvetica" || f.as_ref() == "Arial")
            .cloned()
            .unwrap_or_else(|| {
                available_fonts
                    .first()
                    .cloned()
                    .unwrap_or_else(|| ".SystemUIFont".into())
            });

        let multiline_input = cx.new(|cx| {
            let mut input = InputState::new_multiline(cx);
            input.set_content(initial_sample.content(), cx);
            input
        });

        let singleline_input = cx.new(|cx| {
            let mut input = InputState::new_singleline(cx);
            input.set_content("Single-line text input example", cx);
            input
        });

        let multiline_input_for_dropdown = multiline_input.clone();
        let sample_dropdown = cx.new(|_cx| {
            let options: Vec<(SampleText, &'static str)> =
                SampleText::ALL.iter().map(|s| (*s, s.label())).collect();

            DropdownState::new(
                dropdown("sample-dropdown", options, initial_sample)
                    .on_change(move |sample: SampleText, _window, cx| {
                        multiline_input_for_dropdown.update(cx, |input, cx| {
                            input.set_content(sample.content(), cx);
                        });
                    })
                    .full_width(true),
            )
        });

        let font_options: Vec<(SharedString, SharedString)> = available_fonts
            .iter()
            .map(|f| (f.clone(), f.clone()))
            .collect();

        let default_font_for_dropdown = default_font.clone();
        let font_dropdown = cx.new(|_cx| {
            DropdownState::new(
                dropdown("font-dropdown", font_options, default_font_for_dropdown).full_width(true),
            )
        });

        let weight_options: Vec<(FontWeightOption, &'static str)> = FontWeightOption::ALL
            .iter()
            .map(|w| (*w, w.label()))
            .collect();

        let weight_dropdown = cx.new(|_cx| {
            DropdownState::new(
                dropdown("weight-dropdown", weight_options, FontWeightOption::Normal)
                    .full_width(true),
            )
        });

        let font_size_slider = cx.new(|_cx| {
            Slider::new("font-size", 14.0, 8.0..=48.0)
                .label("Size")
                .step(1.0)
        });

        let line_height_slider = cx.new(|_cx| {
            Slider::new("line-height", 1.4, 1.0..=3.0)
                .label("Line Height")
                .step(0.1)
        });

        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe(
            &font_size_slider,
            |_this, _slider, _event: &SliderChanged, cx| {
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe(
            &line_height_slider,
            |_this, _slider, _event: &SliderChanged, cx| {
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe(
            &font_dropdown,
            |this, dropdown, _event: &DropdownChanged, cx| {
                let selected = dropdown.read(cx).selected.clone();
                this.selected_font = selected;
                cx.notify();
            },
        ));

        subscriptions.push(cx.subscribe(
            &weight_dropdown,
            |this, dropdown, _event: &DropdownChanged, cx| {
                let selected = dropdown.read(cx).selected;
                this.selected_weight = selected.to_font_weight();
                cx.notify();
            },
        ));

        Self {
            multiline_input,
            singleline_input,
            use_multiline: true,
            sample_dropdown,
            font_dropdown,
            weight_dropdown,
            font_size_slider,
            line_height_slider,
            selected_font: default_font,
            selected_weight: FontWeight::NORMAL,
            _subscriptions: subscriptions,
        }
    }

    fn toggle_mode(&mut self, _: &ToggleMode, _window: &mut Window, cx: &mut Context<Self>) {
        self.use_multiline = !self.use_multiline;
        cx.notify();
    }

    fn active_input(&self) -> &Entity<InputState> {
        if self.use_multiline {
            &self.multiline_input
        } else {
            &self.singleline_input
        }
    }
}

impl Focusable for InputSandbox {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.active_input().focus_handle(cx)
    }
}

impl Render for InputSandbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_input = self.active_input().clone();
        let input_state = active_input.read(cx);
        let content = input_state.content().to_string();
        let selected_range = input_state.selected_range().clone();
        let cursor_offset = input_state.cursor_offset();
        let char_count = content.chars().count();
        let line_count = content.lines().count().max(1);

        let focus_handle = active_input.focus_handle(cx);

        let font_size = self.font_size_slider.read(cx).value();
        let line_height = self.line_height_slider.read(cx).value();
        let selected_font = self.selected_font.clone();
        let selected_weight = self.selected_weight;

        let multiline_state = self.multiline_input.read(cx);
        let top_gradient_opacity =
            (multiline_state.distance_from_top() / px(SCROLL_GRADIENT_HEIGHT)).clamp(0.0, 1.0);
        let bottom_gradient_opacity =
            (multiline_state.distance_from_bottom() / px(SCROLL_GRADIENT_HEIGHT)).clamp(0.0, 1.0);

        let multiline_focus = self.multiline_input.focus_handle(cx);
        let singleline_focus = self.singleline_input.focus_handle(cx);

        div()
            .id("input-sandbox")
            .key_context("InputSandbox")
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::toggle_mode))
            .flex()
            .flex_row()
            .bg(ThemeGradient::DarkToBlue.gradient())
            .text_color(Theme::Foreground.hsla())
            .size_full()
            // Left panel - Content area
            .child(
                div()
                    .relative()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .p_4()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .when(self.use_multiline, |this| {
                                this.child(
                                    text_area(&self.multiline_input, cx)
                                        .size_full()
                                        .text_base()
                                        .font_family(selected_font.clone())
                                        .font_weight(selected_weight)
                                        .text_size(px(font_size))
                                        .line_height(DefiniteLength::Fraction(line_height))
                                        .selection_color(Theme::AccentLightPink.hsla().alpha(0.9))
                                        .cursor_color(Theme::AccentLightBlue.hsla()),
                                )
                            })
                            .when(!self.use_multiline, |this| {
                                this.child(
                                    div().flex().items_center().h(px(40.)).child(
                                        input(&self.singleline_input, cx)
                                            .size_full()
                                            .text_base()
                                            .font_family(selected_font.clone())
                                            .font_weight(selected_weight)
                                            .text_size(px(font_size))
                                            .line_height(DefiniteLength::Fraction(line_height))
                                            .selection_color(
                                                Theme::AccentDarkPink.hsla().alpha(0.9),
                                            )
                                            .cursor_color(Theme::AccentLightBlue.hsla()),
                                    ),
                                )
                            }),
                    )
                    .when(self.use_multiline, |this| {
                        this.child(
                            div()
                                .absolute()
                                .w_full()
                                .h(px(SCROLL_GRADIENT_HEIGHT))
                                .top_0()
                                .left_0()
                                .bg(ThemeGradient::BlackToTransparent.gradient())
                                .opacity(top_gradient_opacity),
                        )
                        .child(
                            div()
                                .absolute()
                                .w_full()
                                .h(px(SCROLL_GRADIENT_HEIGHT))
                                .bottom_0()
                                .left_0()
                                .bg(ThemeGradient::TransparentToBlue.gradient())
                                .opacity(bottom_gradient_opacity),
                        )
                    }),
            )
            // Right panel - Sidebar
            .child(
                div()
                    .id("sidebar")
                    .w(px(240.))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    // Mode toggle section
                    .child(
                        sidebar_section(None).child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    toggle_button("single-btn", "Single-line", !self.use_multiline)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            if this.use_multiline {
                                                this.toggle_mode(&ToggleMode, window, cx);
                                            }
                                        })),
                                )
                                .child(
                                    toggle_button("multi-btn", "Multi-line", self.use_multiline)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            if !this.use_multiline {
                                                this.toggle_mode(&ToggleMode, window, cx);
                                            }
                                        })),
                                ),
                        ),
                    )
                    // Sample text selector (only in multiline mode)
                    .when(self.use_multiline, |this| {
                        this.child(
                            sidebar_section("Sample Text").child(self.sample_dropdown.clone()),
                        )
                    })
                    // Font section
                    .child(sidebar_section("Font Family").child(self.font_dropdown.clone()))
                    .child(sidebar_section("Font Weight").child(self.weight_dropdown.clone()))
                    // Typography section
                    .child(
                        sidebar_section("Typography")
                            .child(self.font_size_slider.clone())
                            .child(self.line_height_slider.clone()),
                    )
                    // Stats section
                    .child(
                        sidebar_section("Statistics")
                            .child(stat_row("Cursor", format!("{}", cursor_offset)))
                            .child(stat_row(
                                "Selection",
                                format!("{}..{}", selected_range.start, selected_range.end),
                            ))
                            .child(stat_row("Characters", format!("{}", char_count)))
                            .child(stat_row("Lines", format!("{}", line_count)))
                            .child(stat_row("Bytes", format!("{}", content.len()))),
                    )
                    // Focus state section
                    .child(
                        sidebar_section("Focus State")
                            .child(stat_row(
                                "Multi-line",
                                if multiline_focus.is_focused(window) {
                                    "focused"
                                } else {
                                    "—"
                                },
                            ))
                            .child(stat_row(
                                "Single-line",
                                if singleline_focus.is_focused(window) {
                                    "focused"
                                } else {
                                    "—"
                                },
                            )),
                    )
                    // Keybindings section
                    .child(
                        sidebar_section("Keybindings")
                            .child(key_row("Ctrl+T", "Toggle mode"))
                            .child(key_row("Cmd+Z", "Undo"))
                            .child(key_row("Cmd+Shift+Z", "Redo"))
                            .child(key_row("Cmd+A", "Select all"))
                            .child(key_row("Cmd+C", "Copy"))
                            .child(key_row("Cmd+X", "Cut"))
                            .child(key_row("Cmd+V", "Paste"))
                            .child(key_row("Alt+←/→", "Word nav"))
                            .child(key_row("Cmd+←/→", "Line start/end"))
                            .child(key_row("Cmd+↑/↓", "Doc start/end")),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpuikit::init(cx);
        bind_input_keys(cx, None);

        cx.bind_keys([KeyBinding::new("ctrl-t", ToggleMode, None)]);

        let bounds = Bounds::centered(None, size(px(1100.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| InputSandbox::new(window, cx));
                let focus_handle = view.read(cx).active_input().focus_handle(cx);
                window.focus(&focus_handle);
                view
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
