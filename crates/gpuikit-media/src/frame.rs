//! Video frame representation for cross-platform rendering.
//!
//! This module provides [`VideoFrame`], which wraps platform-specific
//! video frame data for efficient rendering:
//!
//! - **macOS**: Uses `CVPixelBuffer` for hardware-accelerated YCbCr→RGB conversion
//! - **Other platforms**: Uses RGBA pixel data converted to `RenderImage`

use std::sync::Arc;

use gpui::{DevicePixels, RenderImage, Size};

#[cfg(target_os = "macos")]
use core_video::pixel_buffer::CVPixelBuffer;

use crate::Timestamp;

/// A video frame that can be rendered efficiently.
///
/// On macOS, this wraps a `CVPixelBuffer` which enables zero-copy
/// hardware-accelerated rendering through Metal. On other platforms,
/// this wraps RGBA pixel data that gets uploaded to a texture atlas.
#[derive(Clone)]
pub struct VideoFrame {
    /// The frame data (platform-specific).
    pub(crate) data: VideoFrameData,

    /// Width of the frame in pixels.
    pub width: u32,

    /// Height of the frame in pixels.
    pub height: u32,

    /// Presentation timestamp for A/V sync.
    pub pts: Timestamp,

    /// Duration this frame should be displayed.
    pub duration: Option<Timestamp>,
}

/// Platform-specific frame data storage.
#[derive(Clone)]
pub(crate) enum VideoFrameData {
    /// macOS CVPixelBuffer for hardware-accelerated rendering.
    #[cfg(target_os = "macos")]
    CoreVideo(CVPixelBuffer),

    /// RGBA pixel data for software rendering.
    Rgba(Arc<RgbaFrame>),

    /// Pre-converted RenderImage ready for display.
    RenderImage(Arc<RenderImage>),
}

/// RGBA frame data for non-macOS platforms or software fallback.
#[derive(Clone)]
pub struct RgbaFrame {
    /// RGBA pixel data (4 bytes per pixel).
    pub pixels: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl VideoFrame {
    /// Create a new video frame from a macOS CVPixelBuffer.
    ///
    /// This is the optimal path on macOS as it enables hardware-accelerated
    /// YCbCr→RGB conversion in the Metal shader pipeline.
    #[cfg(target_os = "macos")]
    pub fn from_cv_pixel_buffer(buffer: CVPixelBuffer) -> Self {
        let width = buffer.get_width() as u32;
        let height = buffer.get_height() as u32;
        Self {
            data: VideoFrameData::CoreVideo(buffer),
            width,
            height,
            pts: Timestamp::ZERO,
            duration: None,
        }
    }

    /// Create a new video frame from a macOS CVPixelBuffer with timing info.
    #[cfg(target_os = "macos")]
    pub fn from_cv_pixel_buffer_with_pts(
        buffer: CVPixelBuffer,
        pts: Timestamp,
        duration: Option<Timestamp>,
    ) -> Self {
        let width = buffer.get_width() as u32;
        let height = buffer.get_height() as u32;
        Self {
            data: VideoFrameData::CoreVideo(buffer),
            width,
            height,
            pts,
            duration,
        }
    }

    /// Create a new video frame from RGBA pixel data.
    ///
    /// The pixel data should be in RGBA format with 4 bytes per pixel,
    /// arranged in row-major order from top-left to bottom-right.
    pub fn from_rgba(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        debug_assert_eq!(
            pixels.len(),
            (width * height * 4) as usize,
            "RGBA pixel data size mismatch"
        );

        Self {
            data: VideoFrameData::Rgba(Arc::new(RgbaFrame {
                pixels,
                width,
                height,
            })),
            width,
            height,
            pts: Timestamp::ZERO,
            duration: None,
        }
    }

    /// Create a new video frame from RGBA pixel data with timing info.
    pub fn from_rgba_with_pts(
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        pts: Timestamp,
        duration: Option<Timestamp>,
    ) -> Self {
        debug_assert_eq!(
            pixels.len(),
            (width * height * 4) as usize,
            "RGBA pixel data size mismatch"
        );

        Self {
            data: VideoFrameData::Rgba(Arc::new(RgbaFrame {
                pixels,
                width,
                height,
            })),
            width,
            height,
            pts,
            duration,
        }
    }

    /// Create a video frame from a pre-converted RenderImage.
    pub fn from_render_image(image: Arc<RenderImage>, width: u32, height: u32) -> Self {
        Self {
            data: VideoFrameData::RenderImage(image),
            width,
            height,
            pts: Timestamp::ZERO,
            duration: None,
        }
    }

    /// Create a video frame from a pre-converted RenderImage with timing info.
    pub fn from_render_image_with_pts(
        image: Arc<RenderImage>,
        width: u32,
        height: u32,
        pts: Timestamp,
        duration: Option<Timestamp>,
    ) -> Self {
        Self {
            data: VideoFrameData::RenderImage(image),
            width,
            height,
            pts,
            duration,
        }
    }

    /// Returns the size of the frame in pixels.
    pub fn size(&self) -> Size<u32> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns the size as DevicePixels for layout calculations.
    pub fn size_device_pixels(&self) -> Size<DevicePixels> {
        Size {
            width: DevicePixels::from(self.width as i32),
            height: DevicePixels::from(self.height as i32),
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

    /// Set the presentation timestamp.
    pub fn with_pts(mut self, pts: Timestamp) -> Self {
        self.pts = pts;
        self
    }

    /// Set the frame duration.
    pub fn with_duration(mut self, duration: Timestamp) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Check if this frame uses hardware-accelerated rendering.
    pub fn is_hardware_accelerated(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            matches!(self.data, VideoFrameData::CoreVideo(_))
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Get the underlying CVPixelBuffer if available (macOS only).
    #[cfg(target_os = "macos")]
    pub fn as_cv_pixel_buffer(&self) -> Option<&CVPixelBuffer> {
        match &self.data {
            VideoFrameData::CoreVideo(buffer) => Some(buffer),
            _ => None,
        }
    }

    /// Convert to a RenderImage for software rendering.
    ///
    /// On macOS with CVPixelBuffer data, this will perform a CPU-side
    /// conversion which is slower than using the hardware path.
    pub fn to_render_image(&self) -> Option<Arc<RenderImage>> {
        match &self.data {
            VideoFrameData::RenderImage(image) => Some(image.clone()),
            VideoFrameData::Rgba(rgba) => {
                let image = rgba_to_render_image(&rgba.pixels, rgba.width, rgba.height)?;
                Some(Arc::new(image))
            }
            #[cfg(target_os = "macos")]
            VideoFrameData::CoreVideo(buffer) => {
                let rgba = cv_pixel_buffer_to_rgba(buffer)?;
                let image = rgba_to_render_image(&rgba, self.width, self.height)?;
                Some(Arc::new(image))
            }
        }
    }
}

#[cfg(target_os = "macos")]
impl From<CVPixelBuffer> for VideoFrame {
    fn from(buffer: CVPixelBuffer) -> Self {
        Self::from_cv_pixel_buffer(buffer)
    }
}

/// Convert RGBA pixels to a RenderImage.
fn rgba_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{Frame, RgbaImage};
    use smallvec::SmallVec;

    let image = RgbaImage::from_raw(width, height, pixels.to_vec())?;
    let frame = Frame::new(image);
    let frames: SmallVec<[Frame; 1]> = smallvec::smallvec![frame];
    Some(RenderImage::new(frames))
}

/// Convert a CVPixelBuffer to RGBA pixels (macOS only, slow path).
#[cfg(target_os = "macos")]
fn cv_pixel_buffer_to_rgba(buffer: &CVPixelBuffer) -> Option<Vec<u8>> {
    use core_video::pixel_buffer::kCVPixelFormatType_32BGRA;
    use core_video::r#return::kCVReturnSuccess;

    let width = buffer.get_width();
    let height = buffer.get_height();
    let format = buffer.get_pixel_format();

    unsafe {
        if buffer.lock_base_address(0) != kCVReturnSuccess {
            return None;
        }

        let result = match format {
            kCVPixelFormatType_32BGRA => {
                let base_address = buffer.get_base_address();
                let bytes_per_row = buffer.get_bytes_per_row();
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);

                for y in 0..height {
                    let row_ptr = (base_address as *const u8).add(y * bytes_per_row);
                    for x in 0..width {
                        let pixel_ptr = row_ptr.add((x * 4) as usize);
                        let b = *pixel_ptr;
                        let g = *pixel_ptr.add(1);
                        let r = *pixel_ptr.add(2);
                        let a = *pixel_ptr.add(3);
                        rgba.extend_from_slice(&[r, g, b, a]);
                    }
                }
                Some(rgba)
            }
            _ => {
                log::warn!(
                    "Unsupported CVPixelBuffer format for RGBA conversion: {}",
                    format
                );
                None
            }
        };

        buffer.unlock_base_address(0);
        result
    }
}

/// A buffer pool for efficient frame allocation (macOS only).
#[cfg(target_os = "macos")]
pub struct FrameBufferPool {
    pool: core_video::pixel_buffer_pool::CVPixelBufferPool,
    width: u32,
    height: u32,
}

#[cfg(target_os = "macos")]
impl FrameBufferPool {
    /// Create a new buffer pool for frames of the given size.
    ///
    /// Uses NV12 format for efficient YCbCr storage.
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        use core_foundation::{base::TCFType, number::CFNumber, string::CFString};
        use core_video::{
            pixel_buffer, pixel_buffer::kCVPixelFormatType_420YpCbCr8BiPlanarFullRange,
            pixel_buffer_io_surface::kCVPixelBufferIOSurfaceCoreAnimationCompatibilityKey,
            pixel_buffer_pool,
        };

        let width_key: CFString =
            unsafe { CFString::wrap_under_get_rule(pixel_buffer::kCVPixelBufferWidthKey) };
        let height_key: CFString =
            unsafe { CFString::wrap_under_get_rule(pixel_buffer::kCVPixelBufferHeightKey) };
        let animation_key: CFString = unsafe {
            CFString::wrap_under_get_rule(kCVPixelBufferIOSurfaceCoreAnimationCompatibilityKey)
        };
        let format_key: CFString = unsafe {
            CFString::wrap_under_get_rule(pixel_buffer::kCVPixelBufferPixelFormatTypeKey)
        };

        let yes: CFNumber = 1.into();
        let width_cf: CFNumber = (width as i32).into();
        let height_cf: CFNumber = (height as i32).into();
        let format: CFNumber = (kCVPixelFormatType_420YpCbCr8BiPlanarFullRange as i64).into();

        let buffer_attributes = core_foundation::dictionary::CFDictionary::from_CFType_pairs(&[
            (width_key, width_cf.into_CFType()),
            (height_key, height_cf.into_CFType()),
            (animation_key, yes.into_CFType()),
            (format_key, format.into_CFType()),
        ]);

        let pool = pixel_buffer_pool::CVPixelBufferPool::new(None, Some(&buffer_attributes))
            .map_err(|cv_return| {
                anyhow::anyhow!(
                    "Failed to create pixel buffer pool: CVReturn({})",
                    cv_return
                )
            })?;

        Ok(Self {
            pool,
            width,
            height,
        })
    }

    /// Get a pixel buffer from the pool.
    pub fn get_buffer(&self) -> anyhow::Result<CVPixelBuffer> {
        self.pool.create_pixel_buffer().map_err(|cv_return| {
            anyhow::anyhow!("Failed to create pixel buffer: CVReturn({})", cv_return)
        })
    }

    /// Get the configured width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the configured height.
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pts", &self.pts)
            .field("duration", &self.duration)
            .field("hardware_accelerated", &self.is_hardware_accelerated())
            .finish()
    }
}
