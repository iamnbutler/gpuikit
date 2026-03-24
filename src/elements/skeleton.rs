//! Skeleton loading placeholder component for gpuikit

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, px, Animation, AnimationExt, App, Div, IntoElement, Pixels,
    RenderOnce, Styled, Window,
};
use std::time::Duration;

/// Create a basic skeleton placeholder.
pub fn skeleton() -> Skeleton {
    Skeleton::new()
}

/// Create a text line skeleton placeholder.
pub fn skeleton_text() -> Skeleton {
    Skeleton::new().h(px(16.0)).w(px(200.0)).rounded(px(4.0))
}

/// Create a circular avatar skeleton placeholder.
pub fn skeleton_avatar() -> Skeleton {
    Skeleton::new().size(px(40.0)).circle()
}

/// Create a card-shaped skeleton placeholder.
pub fn skeleton_card() -> Skeleton {
    Skeleton::new()
        .w(px(300.0))
        .h(px(150.0))
        .rounded(px(8.0))
}

#[derive(IntoElement)]
pub struct Skeleton {
    width: Option<Pixels>,
    height: Option<Pixels>,
    full_width: bool,
    corner_radius: Option<Pixels>,
    is_circle: bool,
    animated: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            full_width: false,
            corner_radius: Some(px(4.0)),
            is_circle: false,
            animated: true,
        }
    }

    /// Set the width of the skeleton.
    pub fn w(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height of the skeleton.
    pub fn h(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Set both width and height to the same value.
    pub fn size(mut self, size: Pixels) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    /// Make the skeleton take full width of its container.
    pub fn w_full(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Set the corner radius.
    pub fn rounded(mut self, radius: Pixels) -> Self {
        self.corner_radius = Some(radius);
        self.is_circle = false;
        self
    }

    /// Make the skeleton circular (rounded_full).
    pub fn circle(mut self) -> Self {
        self.is_circle = true;
        self.corner_radius = None;
        self
    }

    /// Enable or disable the pulse animation.
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

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.surface_secondary();
        let animated = self.animated;

        div()
            .bg(bg_color)
            .when(self.full_width, |this: Div| this.w_full())
            .when_some(self.width, |this: Div, w| this.w(w))
            .when_some(self.height, |this: Div, h| this.h(h))
            .when(self.is_circle, |this: Div| this.rounded_full())
            .when_some(self.corner_radius, |this: Div, r| this.rounded(r))
            .with_animation(
                "skeleton-pulse",
                Animation::new(Duration::from_millis(1500)).repeat(),
                move |el, delta| {
                    if animated {
                        // Pulse between 0.4 and 1.0 opacity using sine wave
                        let opacity =
                            0.4 + 0.6 * (1.0 - (delta * std::f32::consts::PI * 2.0).cos()) / 2.0;
                        el.opacity(opacity)
                    } else {
                        el
                    }
                },
            )
    }
}
