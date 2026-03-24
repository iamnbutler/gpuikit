use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, px, App, Animation, AnimationExt, Div, IntoElement, ParentElement,
    Pixels, RenderOnce, Styled, Window,
};
use std::time::Duration;

pub fn skeleton() -> Skeleton {
    Skeleton::new()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    #[default]
    Rectangle,
    Circle,
    Text,
}

#[derive(IntoElement)]
pub struct Skeleton {
    shape: SkeletonShape,
    width: Option<Pixels>,
    height: Option<Pixels>,
    lines: usize,
    animated: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            shape: SkeletonShape::Rectangle,
            width: None,
            height: None,
            lines: 3,
            animated: true,
        }
    }

    pub fn shape(mut self, shape: SkeletonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn rectangle(mut self) -> Self {
        self.shape = SkeletonShape::Rectangle;
        self
    }

    pub fn circle(mut self) -> Self {
        self.shape = SkeletonShape::Circle;
        self
    }

    pub fn text(mut self) -> Self {
        self.shape = SkeletonShape::Text;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(IntoElement, Clone)]
struct SkeletonBlock {
    width: Option<Pixels>,
    height: Pixels,
    rounded_full: bool,
    opacity: f32,
}

impl SkeletonBlock {
    fn new(height: Pixels) -> Self {
        Self {
            width: None,
            height,
            rounded_full: false,
            opacity: 1.0,
        }
    }

    fn with_width(mut self, width: Option<Pixels>) -> Self {
        self.width = width;
        self
    }

    fn with_rounded_full(mut self) -> Self {
        self.rounded_full = true;
        self
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }
}

impl RenderOnce for SkeletonBlock {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let base_color = theme.surface_secondary();

        div()
            .h(self.height)
            .when_some(self.width, |el: Div, w: Pixels| el.w(w))
            .when(self.width.is_none(), |el: Div| el.w_full())
            .when(self.rounded_full, |el: Div| el.rounded_full())
            .when(!self.rounded_full, |el: Div| el.rounded(px(4.0)))
            .bg(base_color.opacity(self.opacity))
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match self.shape {
            SkeletonShape::Rectangle => {
                let width = self.width;
                let height = self.height.unwrap_or(px(20.0));

                let block = SkeletonBlock::new(height).with_width(width);

                if self.animated {
                    div().child(
                        block.with_animation(
                            "skeleton-shimmer",
                            Animation::new(Duration::from_millis(1500)).repeat(),
                            |mut block: SkeletonBlock, delta: f32| {
                                let opacity = 0.4 + 0.3 * (delta * std::f32::consts::PI * 2.0).sin();
                                block.set_opacity(opacity);
                                block
                            },
                        ),
                    )
                } else {
                    div().child(block)
                }
            }
            SkeletonShape::Circle => {
                let size = self.width.or(self.height).unwrap_or(px(40.0));

                let block = SkeletonBlock::new(size)
                    .with_width(Some(size))
                    .with_rounded_full();

                if self.animated {
                    div().child(
                        block.with_animation(
                            "skeleton-shimmer-circle",
                            Animation::new(Duration::from_millis(1500)).repeat(),
                            |mut block: SkeletonBlock, delta: f32| {
                                let opacity = 0.4 + 0.3 * (delta * std::f32::consts::PI * 2.0).sin();
                                block.set_opacity(opacity);
                                block
                            },
                        ),
                    )
                } else {
                    div().child(block)
                }
            }
            SkeletonShape::Text => {
                let width = self.width;
                let line_height = self.height.unwrap_or(px(16.0));
                let lines = self.lines;
                let animated = self.animated;

                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .when_some(width, |el: Div, w: Pixels| el.w(w))
                    .when(width.is_none(), |el: Div| el.w_full())
                    .children((0..lines).map(move |i| {
                        let is_last = i == lines - 1;
                        let line_width = if is_last {
                            Some(px(60.0))
                        } else {
                            None
                        };

                        let block = SkeletonBlock::new(line_height).with_width(line_width);

                        if animated {
                            div().w_full().child(
                                block.with_animation(
                                    "skeleton-shimmer-text",
                                    Animation::new(Duration::from_millis(1500)).repeat(),
                                    |mut block: SkeletonBlock, delta: f32| {
                                        let opacity =
                                            0.4 + 0.3 * (delta * std::f32::consts::PI * 2.0).sin();
                                        block.set_opacity(opacity);
                                        block
                                    },
                                ),
                            )
                        } else {
                            div().w_full().child(block)
                        }
                    }))
            }
        }
    }
}
