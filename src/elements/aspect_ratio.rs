//! AspectRatio component for gpuikit
//!
//! A container component that maintains a specific aspect ratio for its content.
//! Useful for images, videos, and other media that need consistent proportions.

use gpui::{
    div, prelude::FluentBuilder, relative, AnyElement, App, IntoElement, ParentElement, Pixels,
    RenderOnce, Styled, Window,
};

/// Common aspect ratio presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectRatioPreset {
    /// 1:1 square ratio
    Square,
    /// 4:3 standard ratio (common for photos)
    Standard,
    /// 3:4 portrait ratio
    Portrait,
    /// 16:9 widescreen video ratio
    Video,
    /// 21:9 ultrawide ratio
    Ultrawide,
    /// 2:3 portrait photography ratio
    Portrait23,
    /// 3:2 landscape photography ratio
    Landscape32,
}

impl AspectRatioPreset {
    /// Get the numeric ratio value (width / height)
    pub fn ratio(&self) -> f32 {
        match self {
            AspectRatioPreset::Square => 1.0,
            AspectRatioPreset::Standard => 4.0 / 3.0,
            AspectRatioPreset::Portrait => 3.0 / 4.0,
            AspectRatioPreset::Video => 16.0 / 9.0,
            AspectRatioPreset::Ultrawide => 21.0 / 9.0,
            AspectRatioPreset::Portrait23 => 2.0 / 3.0,
            AspectRatioPreset::Landscape32 => 3.0 / 2.0,
        }
    }
}

/// How the child content should fit within the aspect ratio container
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ObjectFit {
    /// Scale content to fill the container, maintaining aspect ratio.
    /// Content may be clipped.
    #[default]
    Cover,
    /// Scale content to fit entirely within the container, maintaining aspect ratio.
    /// May result in letterboxing.
    Contain,
    /// Stretch content to fill the container exactly.
    /// Aspect ratio of content may be distorted.
    Fill,
    /// Content is not resized. May overflow or leave empty space.
    None,
}

/// A container that maintains a specific aspect ratio for its content.
#[derive(IntoElement)]
pub struct AspectRatio {
    ratio: f32,
    child: Option<AnyElement>,
    object_fit: ObjectFit,
    width: Option<Pixels>,
}

impl AspectRatio {
    /// Create a new AspectRatio container with the given ratio (width / height).
    ///
    /// # Example
    /// ```ignore
    /// aspect_ratio(16.0 / 9.0)
    ///     .child(video_player)
    /// ```
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.max(0.001), // Prevent division by zero
            child: None,
            object_fit: ObjectFit::default(),
            width: None,
        }
    }

    /// Create an AspectRatio from a preset ratio.
    pub fn preset(preset: AspectRatioPreset) -> Self {
        Self::new(preset.ratio())
    }

    /// Set the child element to display within the aspect ratio container.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }

    /// Set how the child content should fit within the container.
    pub fn object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self
    }

    /// Set a fixed width for the container. Height will be calculated from the aspect ratio.
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the aspect ratio (width / height).
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.max(0.001);
        self
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl RenderOnce for AspectRatio {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Use the padding-bottom technique to maintain aspect ratio
        // This works by setting the height to 0 and using padding-bottom as a percentage of width
        let padding_percent = (1.0 / self.ratio).clamp(0.0, 10.0);

        let outer = div()
            .relative()
            .overflow_hidden()
            .when_some(self.width, |el, w| el.w(w))
            .when(self.width.is_none(), |el| el.w_full())
            // Use padding-bottom to create the aspect ratio
            .h(relative(0.0))
            .pb(relative(padding_percent));

        let inner = div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .when(matches!(self.object_fit, ObjectFit::Fill), |el| {
                el.size_full()
            })
            .when(matches!(self.object_fit, ObjectFit::Cover), |el| {
                el.size_full().overflow_hidden()
            })
            .when(matches!(self.object_fit, ObjectFit::Contain), |el| {
                el.size_full()
            })
            .when(matches!(self.object_fit, ObjectFit::None), |el| el)
            .when_some(self.child, |el, child| el.child(child));

        outer.child(inner)
    }
}

// Convenience functions

/// Create an AspectRatio container with a custom ratio (width / height).
///
/// # Example
/// ```ignore
/// aspect_ratio(16.0 / 9.0)
///     .child(video_thumbnail)
/// ```
pub fn aspect_ratio(ratio: f32) -> AspectRatio {
    AspectRatio::new(ratio)
}

/// Create a 1:1 square aspect ratio container.
///
/// # Example
/// ```ignore
/// aspect_ratio_square()
///     .child(avatar)
/// ```
pub fn aspect_ratio_square() -> AspectRatio {
    AspectRatio::preset(AspectRatioPreset::Square)
}

/// Create a 16:9 widescreen video aspect ratio container.
///
/// # Example
/// ```ignore
/// aspect_ratio_video()
///     .child(video_player)
/// ```
pub fn aspect_ratio_video() -> AspectRatio {
    AspectRatio::preset(AspectRatioPreset::Video)
}

/// Create a 3:4 portrait aspect ratio container.
///
/// # Example
/// ```ignore
/// aspect_ratio_portrait()
///     .child(profile_image)
/// ```
pub fn aspect_ratio_portrait() -> AspectRatio {
    AspectRatio::preset(AspectRatioPreset::Portrait)
}

/// Create a 4:3 standard aspect ratio container.
///
/// # Example
/// ```ignore
/// aspect_ratio_standard()
///     .child(photo)
/// ```
pub fn aspect_ratio_standard() -> AspectRatio {
    AspectRatio::preset(AspectRatioPreset::Standard)
}

/// Create a 21:9 ultrawide aspect ratio container.
///
/// # Example
/// ```ignore
/// aspect_ratio_ultrawide()
///     .child(cinematic_content)
/// ```
pub fn aspect_ratio_ultrawide() -> AspectRatio {
    AspectRatio::preset(AspectRatioPreset::Ultrawide)
}
