//! Loading indicator component for gpuikit

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, rems, Animation, AnimationExt, App, Hsla, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, Window,
};
use std::time::Duration;

pub fn loading_indicator() -> LoadingIndicator {
    LoadingIndicator::new()
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingIndicatorVariant {
    #[default]
    Dots,
    Ellipsis,
    Dash,
    Star,
}

impl LoadingIndicatorVariant {
    fn frames(&self) -> &'static [&'static str] {
        match self {
            LoadingIndicatorVariant::Dots => &[".  ", ".. ", "..."],
            LoadingIndicatorVariant::Ellipsis => &["   ", ".  ", ".. ", "...", ".. ", ".  "],
            LoadingIndicatorVariant::Dash => &["-", "\\", "|", "/"],
            LoadingIndicatorVariant::Star => &["❊", "❊", "✳︎", "※"],
        }
    }

    fn duration(&self) -> Duration {
        match self {
            LoadingIndicatorVariant::Dots => Duration::from_millis(1500),
            LoadingIndicatorVariant::Ellipsis => Duration::from_millis(1800),
            LoadingIndicatorVariant::Dash => Duration::from_millis(400),
            LoadingIndicatorVariant::Star => Duration::from_millis(1000),
        }
    }

    fn animation_id(&self) -> &'static str {
        match self {
            LoadingIndicatorVariant::Dots => "loading-dots",
            LoadingIndicatorVariant::Ellipsis => "loading-ellipsis",
            LoadingIndicatorVariant::Dash => "loading-dash",
            LoadingIndicatorVariant::Star => "loading-star",
        }
    }

    fn char_width(&self) -> usize {
        self.frames()
            .iter()
            .map(|f| f.chars().count())
            .max()
            .unwrap_or(1)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingIndicatorSize {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(IntoElement)]
pub struct LoadingIndicator {
    variant: LoadingIndicatorVariant,
    size: LoadingIndicatorSize,
    color: Option<Hsla>,
}

impl LoadingIndicator {
    pub fn new() -> Self {
        Self {
            variant: LoadingIndicatorVariant::default(),
            size: LoadingIndicatorSize::default(),
            color: None,
        }
    }

    pub fn variant(mut self, variant: LoadingIndicatorVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn dots(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Dots;
        self
    }

    pub fn ellipsis(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Ellipsis;
        self
    }

    pub fn dash(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Dash;
        self
    }

    pub fn star(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Star;
        self
    }

    pub fn size(mut self, size: LoadingIndicatorSize) -> Self {
        self.size = size;
        self
    }

    pub fn xsmall(mut self) -> Self {
        self.size = LoadingIndicatorSize::XSmall;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = LoadingIndicatorSize::Small;
        self
    }

    pub fn medium(mut self) -> Self {
        self.size = LoadingIndicatorSize::Medium;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = LoadingIndicatorSize::Large;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(IntoElement)]
struct LoadingFrame {
    text: SharedString,
}

impl LoadingFrame {
    fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }

    fn set_text(&mut self, text: impl Into<SharedString>) {
        self.text = text.into();
    }
}

impl RenderOnce for LoadingFrame {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.text
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or_else(|| theme.accent());
        let frames: Vec<SharedString> = self
            .variant
            .frames()
            .iter()
            .map(|&s| SharedString::from(s))
            .collect();
        let duration = self.variant.duration();
        let animation_id = self.variant.animation_id();
        let initial_frame = frames[0].clone();

        let size = self.size;
        let char_width = self.variant.char_width();
        let width_rems = char_width as f32 * 0.6;

        div()
            .text_color(color)
            .flex_none()
            .min_w(rems(width_rems))
            .text_center()
            .when(matches!(size, LoadingIndicatorSize::XSmall), |this| {
                this.text_xs()
            })
            .when(matches!(size, LoadingIndicatorSize::Small), |this| {
                this.text_sm()
            })
            .when(matches!(size, LoadingIndicatorSize::Medium), |this| {
                this.text_base()
            })
            .when(matches!(size, LoadingIndicatorSize::Large), |this| {
                this.text_xl()
            })
            .child(LoadingFrame::new(initial_frame).with_animation(
                animation_id,
                Animation::new(duration).repeat(),
                move |mut frame, delta| {
                    let frame_index = (delta * frames.len() as f32) as usize % frames.len();
                    frame.set_text(frames[frame_index].clone());
                    frame
                },
            ))
    }
}
