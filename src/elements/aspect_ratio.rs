use gpui::{
    div, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};

/// Creates a new AspectRatio container with the given ratio (width / height).
///
/// # Example
/// ```
/// use gpuikit::elements::aspect_ratio::aspect_ratio;
///
/// // 16:9 video aspect ratio
/// let video = aspect_ratio(16.0 / 9.0).child(my_content);
///
/// // Using a preset
/// let square = aspect_ratio_square().child(my_content);
/// ```
pub fn aspect_ratio(ratio: f32) -> AspectRatio {
    AspectRatio::new(ratio)
}

/// Creates a square aspect ratio container (1:1).
pub fn aspect_ratio_square() -> AspectRatio {
    AspectRatio::square()
}

/// Creates a video aspect ratio container (16:9).
pub fn aspect_ratio_video() -> AspectRatio {
    AspectRatio::video()
}

/// Creates a photo aspect ratio container (4:3).
pub fn aspect_ratio_photo() -> AspectRatio {
    AspectRatio::photo()
}

/// Common aspect ratio presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectRatioPreset {
    /// 1:1 square ratio
    Square,
    /// 16:9 widescreen video ratio
    Video,
    /// 4:3 traditional photo ratio
    Photo,
    /// 3:2 classic photo/35mm film ratio
    Classic,
    /// 21:9 ultrawide cinema ratio
    Ultrawide,
    /// 9:16 portrait/vertical video ratio
    Portrait,
}

impl AspectRatioPreset {
    /// Returns the ratio value (width / height) for this preset.
    pub fn ratio(self) -> f32 {
        match self {
            AspectRatioPreset::Square => 1.0,
            AspectRatioPreset::Video => 16.0 / 9.0,
            AspectRatioPreset::Photo => 4.0 / 3.0,
            AspectRatioPreset::Classic => 3.0 / 2.0,
            AspectRatioPreset::Ultrawide => 21.0 / 9.0,
            AspectRatioPreset::Portrait => 9.0 / 16.0,
        }
    }
}

/// A container element that maintains a specific aspect ratio.
///
/// The AspectRatio component wraps child content and ensures the container
/// maintains the specified width-to-height ratio regardless of the content.
#[derive(IntoElement)]
pub struct AspectRatio {
    ratio: f32,
    child: Option<AnyElement>,
}

impl AspectRatio {
    /// Creates a new AspectRatio container with the given ratio.
    ///
    /// The ratio is calculated as width divided by height.
    /// For example, 16:9 would be `16.0 / 9.0 = 1.778`.
    pub fn new(ratio: f32) -> Self {
        AspectRatio { ratio, child: None }
    }

    /// Creates a square aspect ratio container (1:1).
    pub fn square() -> Self {
        Self::new(AspectRatioPreset::Square.ratio())
    }

    /// Creates a video aspect ratio container (16:9).
    pub fn video() -> Self {
        Self::new(AspectRatioPreset::Video.ratio())
    }

    /// Creates a photo aspect ratio container (4:3).
    pub fn photo() -> Self {
        Self::new(AspectRatioPreset::Photo.ratio())
    }

    /// Creates a classic photo aspect ratio container (3:2).
    pub fn classic() -> Self {
        Self::new(AspectRatioPreset::Classic.ratio())
    }

    /// Creates an ultrawide aspect ratio container (21:9).
    pub fn ultrawide() -> Self {
        Self::new(AspectRatioPreset::Ultrawide.ratio())
    }

    /// Creates a portrait aspect ratio container (9:16).
    pub fn portrait() -> Self {
        Self::new(AspectRatioPreset::Portrait.ratio())
    }

    /// Sets the ratio using a preset.
    pub fn preset(mut self, preset: AspectRatioPreset) -> Self {
        self.ratio = preset.ratio();
        self
    }

    /// Sets the aspect ratio (width / height).
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    /// Sets the child element to display within the aspect ratio container.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::square()
    }
}

impl RenderOnce for AspectRatio {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let container = div()
            .w_full()
            .aspect_ratio(self.ratio)
            .overflow_hidden();

        match self.child {
            Some(child) => container.child(
                div()
                    .size_full()
                    .child(child),
            ),
            None => container,
        }
    }
}
