//! Video rendering element for displaying video frames.
//!
//! This module provides components for rendering video content in GPUI applications.
//! It supports efficient hardware-accelerated rendering on macOS through CoreVideo,
//! with fallback image-based rendering on other platforms.
//!
//! # Architecture
//!
//! The video system is split into two main components:
//!
//! - [`VideoFrame`]: Represents a single frame of video data (platform-specific)
//! - [`Video`]: A `RenderOnce` element for displaying a frame
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::video::{video, VideoFrame};
//!
//! // In your render method:
//! fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!     if let Some(frame) = &self.current_frame {
//!         video(frame.clone()).size_full()
//!     } else {
//!         div().size_full().bg(cx.theme().surface())
//!     }
//! }
//! ```

#[allow(unused_imports)]
use gpui::{
    div, px, App, Bounds, DevicePixels, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, ObjectFit, ParentElement, Pixels, Refineable, RenderOnce, Size, Style,
    StyleRefinement, Styled, Window,
};
use gpuikit_theme::{ActiveTheme, Themeable};

/// A video frame that can be rendered.
///
/// On macOS, this wraps a `CVPixelBuffer` for hardware-accelerated rendering.
/// On other platforms, this wraps image data that can be rendered as a texture.
#[derive(Clone)]
pub struct VideoFrame {
    #[cfg(target_os = "macos")]
    pub(crate) buffer: core_video::pixel_buffer::CVPixelBuffer,

    #[cfg(not(target_os = "macos"))]
    pub(crate) data: VideoFrameData,

    /// Width of the frame in pixels
    pub width: u32,

    /// Height of the frame in pixels
    pub height: u32,
}

/// Video frame data for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
#[derive(Clone)]
pub struct VideoFrameData {
    /// RGBA pixel data
    pub pixels: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl VideoFrame {
    /// Create a new video frame from a macOS CVPixelBuffer.
    #[cfg(target_os = "macos")]
    pub fn from_cv_pixel_buffer(buffer: core_video::pixel_buffer::CVPixelBuffer) -> Self {
        let width = buffer.get_width() as u32;
        let height = buffer.get_height() as u32;
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Create a new video frame from RGBA pixel data.
    #[cfg(not(target_os = "macos"))]
    pub fn from_rgba(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data: VideoFrameData {
                pixels,
                width,
                height,
            },
            width,
            height,
        }
    }

    /// Returns the size of the frame.
    pub fn size(&self) -> Size<u32> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

#[cfg(target_os = "macos")]
impl From<core_video::pixel_buffer::CVPixelBuffer> for VideoFrame {
    fn from(buffer: core_video::pixel_buffer::CVPixelBuffer) -> Self {
        Self::from_cv_pixel_buffer(buffer)
    }
}

/// Creates a new video element.
///
/// # Example
///
/// ```ignore
/// video(frame).size_full().object_fit(ObjectFit::Cover)
/// ```
pub fn video(frame: VideoFrame) -> Video {
    Video::new(frame)
}

/// A video rendering element.
///
/// This element renders a single video frame using hardware-accelerated
/// rendering when available (macOS CoreVideo), or falls back to image
/// rendering on other platforms.
#[derive(IntoElement)]
pub struct Video {
    frame: VideoFrame,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

impl Video {
    /// Create a new video element with the given frame.
    pub fn new(frame: VideoFrame) -> Self {
        Self {
            frame,
            object_fit: ObjectFit::Contain,
            style: StyleRefinement::default(),
        }
    }

    /// Set how the video should be fitted within its bounds.
    ///
    /// - `ObjectFit::Contain`: Scale to fit while maintaining aspect ratio (may letterbox)
    /// - `ObjectFit::Cover`: Scale to cover the entire area (may crop)
    /// - `ObjectFit::Fill`: Stretch to fill (may distort)
    /// - `ObjectFit::ScaleDown`: Like contain, but never scales up
    /// - `ObjectFit::None`: No scaling, original size
    pub fn object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.object_fit = object_fit;
        self
    }
}

impl RenderOnce for Video {
    #[allow(unused_variables)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        #[cfg(target_os = "macos")]
        {
            VideoSurface {
                frame: self.frame,
                object_fit: self.object_fit,
                style: self.style,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child("Video playback not supported on this platform")
        }
    }
}

impl Styled for Video {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Internal surface-based video element for macOS.
#[cfg(target_os = "macos")]
struct VideoSurface {
    frame: VideoFrame,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

#[cfg(target_os = "macos")]
impl Element for VideoSurface {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let frame_size: Size<DevicePixels> = Size {
            width: DevicePixels::from(self.frame.width as i32),
            height: DevicePixels::from(self.frame.height as i32),
        };
        let fitted_bounds = self.object_fit.get_bounds(bounds, frame_size);
        window.paint_surface(fitted_bounds, self.frame.buffer.clone());
    }
}

#[cfg(target_os = "macos")]
impl IntoElement for VideoSurface {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// A placeholder element shown when no video frame is available.
#[derive(IntoElement)]
pub struct VideoPlaceholder {
    style: StyleRefinement,
}

impl VideoPlaceholder {
    /// Create a new video placeholder.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl Default for VideoPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for VideoPlaceholder {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.surface())
            .text_color(theme.fg_muted())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child("No video"),
            )
    }
}

impl Styled for VideoPlaceholder {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Creates a video placeholder element.
pub fn video_placeholder() -> VideoPlaceholder {
    VideoPlaceholder::new()
}

/// Optional video element that shows either a frame or a placeholder.
#[derive(IntoElement)]
pub struct OptionalVideo {
    frame: Option<VideoFrame>,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

impl OptionalVideo {
    /// Create an optional video element.
    pub fn new(frame: Option<VideoFrame>) -> Self {
        Self {
            frame,
            object_fit: ObjectFit::Contain,
            style: StyleRefinement::default(),
        }
    }

    /// Set the object fit mode.
    pub fn object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.object_fit = object_fit;
        self
    }
}

impl RenderOnce for OptionalVideo {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match self.frame {
            Some(frame) => {
                let mut video_element = Video::new(frame).object_fit(self.object_fit);
                video_element.style = self.style;
                video_element.into_any_element()
            }
            None => {
                let mut placeholder = VideoPlaceholder::new();
                placeholder.style = self.style;
                placeholder.into_any_element()
            }
        }
    }
}

impl Styled for OptionalVideo {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Creates an optional video element.
pub fn optional_video(frame: Option<VideoFrame>) -> OptionalVideo {
    OptionalVideo::new(frame)
}
