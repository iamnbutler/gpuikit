//! Media source trait for decoder implementations.
//!
//! This module defines the [`MediaSource`] trait which provides a common
//! interface for video and audio decoders. Implementations can wrap:
//!
//! - FFmpeg (via ffmpeg-sidecar or ffmpeg-next)
//! - Platform decoders (AVFoundation on macOS, MediaFoundation on Windows)
//! - Web codecs (in browser environments)
//! - Test/mock sources for development
//!
//! # Implementing a MediaSource
//!
//! ```ignore
//! use gpuikit_media::source::{MediaSource, FrameInfo, MediaInfo, MediaSourceEvent};
//! use gpuikit_media::{VideoFrame, Timestamp, MediaResult};
//!
//! struct MyDecoder { /* ... */ }
//!
//! impl MediaSource for MyDecoder {
//!     fn media_info(&self) -> &MediaInfo {
//!         &self.info
//!     }
//!
//!     fn read_video_frame(&mut self) -> MediaResult<Option<VideoFrame>> {
//!         // Decode and return the next video frame
//!     }
//!
//!     fn read_audio_samples(&mut self, buffer: &mut [f32]) -> MediaResult<usize> {
//!         // Decode audio samples into the buffer
//!     }
//!
//!     // ... other methods
//! }
//! ```

use std::path::Path;

use crate::frame::VideoFrame;
use crate::{MediaError, MediaResult, Timestamp};

/// Information about a media stream (video or audio).
#[derive(Debug, Clone)]
pub enum StreamInfo {
    /// Video stream information.
    Video(VideoStreamInfo),
    /// Audio stream information.
    Audio(AudioStreamInfo),
    /// Subtitle or other stream type.
    Other {
        /// Stream index.
        index: usize,
        /// Codec name if known.
        codec: Option<String>,
    },
}

/// Information about a video stream.
#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
    /// Stream index in the container.
    pub index: usize,
    /// Video codec name (e.g., "h264", "vp9", "av1").
    pub codec: String,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Frame rate as frames per second.
    pub frame_rate: f64,
    /// Pixel format (e.g., "yuv420p", "nv12", "rgb24").
    pub pixel_format: Option<String>,
    /// Bit depth per channel.
    pub bit_depth: Option<u8>,
    /// Color space.
    pub color_space: Option<String>,
    /// Total number of frames if known.
    pub frame_count: Option<u64>,
    /// Stream duration if known.
    pub duration: Option<Timestamp>,
    /// Average bitrate in bits per second.
    pub bitrate: Option<u64>,
}

/// Information about an audio stream.
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    /// Stream index in the container.
    pub index: usize,
    /// Audio codec name (e.g., "aac", "mp3", "opus").
    pub codec: String,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
    /// Channel layout description (e.g., "stereo", "5.1").
    pub channel_layout: Option<String>,
    /// Sample format (e.g., "f32", "s16").
    pub sample_format: Option<String>,
    /// Bit depth per sample.
    pub bit_depth: Option<u8>,
    /// Stream duration if known.
    pub duration: Option<Timestamp>,
    /// Average bitrate in bits per second.
    pub bitrate: Option<u64>,
}

/// Overall information about a media file/stream.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// Container format (e.g., "mp4", "mkv", "webm").
    pub format: String,
    /// Total duration of the media.
    pub duration: Option<Timestamp>,
    /// Overall bitrate in bits per second.
    pub bitrate: Option<u64>,
    /// List of streams in the media.
    pub streams: Vec<StreamInfo>,
    /// Metadata tags (title, artist, etc.).
    pub metadata: Vec<(String, String)>,
    /// Whether the media is seekable.
    pub seekable: bool,
    /// Whether the media is a live stream.
    pub is_live: bool,
}

impl MediaInfo {
    /// Create empty media info (for sources that don't provide metadata).
    pub fn empty() -> Self {
        Self {
            format: String::new(),
            duration: None,
            bitrate: None,
            streams: Vec::new(),
            metadata: Vec::new(),
            seekable: false,
            is_live: false,
        }
    }

    /// Find the first video stream.
    pub fn video_stream(&self) -> Option<&VideoStreamInfo> {
        self.streams.iter().find_map(|s| match s {
            StreamInfo::Video(info) => Some(info),
            _ => None,
        })
    }

    /// Find the first audio stream.
    pub fn audio_stream(&self) -> Option<&AudioStreamInfo> {
        self.streams.iter().find_map(|s| match s {
            StreamInfo::Audio(info) => Some(info),
            _ => None,
        })
    }

    /// Check if the media has video.
    pub fn has_video(&self) -> bool {
        self.video_stream().is_some()
    }

    /// Check if the media has audio.
    pub fn has_audio(&self) -> bool {
        self.audio_stream().is_some()
    }

    /// Get a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    /// Get the title metadata if present.
    pub fn title(&self) -> Option<&str> {
        self.get_metadata("title")
    }

    /// Get the artist/author metadata if present.
    pub fn artist(&self) -> Option<&str> {
        self.get_metadata("artist")
            .or_else(|| self.get_metadata("author"))
    }
}

/// Information about a decoded frame.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// Presentation timestamp.
    pub pts: Timestamp,
    /// Decode timestamp (may differ from pts for B-frames).
    pub dts: Option<Timestamp>,
    /// Duration of this frame.
    pub duration: Option<Timestamp>,
    /// Frame number in sequence.
    pub frame_number: u64,
    /// Whether this is a keyframe.
    pub is_keyframe: bool,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            pts: Timestamp::ZERO,
            dts: None,
            duration: None,
            frame_number: 0,
            is_keyframe: false,
        }
    }
}

/// Events emitted by media sources.
#[derive(Debug, Clone)]
pub enum MediaSourceEvent {
    /// Media info has been parsed and is available.
    MediaInfoReady(MediaInfo),
    /// Buffering started (with buffer percentage 0-100).
    BufferingStarted,
    /// Buffering progress update.
    BufferingProgress(u8),
    /// Buffering completed, playback can resume.
    BufferingComplete,
    /// End of stream reached.
    EndOfStream,
    /// An error occurred.
    Error(String),
    /// Seek completed.
    SeekComplete(Timestamp),
}

/// Trait for media source implementations.
///
/// A media source provides decoded video frames and/or audio samples
/// from some underlying media (file, network stream, etc.).
pub trait MediaSource {
    /// Get information about the media.
    ///
    /// This should return cached info after the first call.
    fn media_info(&self) -> &MediaInfo;

    /// Read the next video frame.
    ///
    /// Returns `Ok(None)` when end of stream is reached.
    /// Returns `Err` on decode errors.
    fn read_video_frame(&mut self) -> MediaResult<Option<VideoFrame>>;

    /// Read audio samples into the provided buffer.
    ///
    /// Returns the number of samples written (per channel).
    /// Returns 0 when end of stream is reached.
    fn read_audio_samples(&mut self, buffer: &mut [f32]) -> MediaResult<usize>;

    /// Seek to the specified timestamp.
    ///
    /// Seeking is approximate for most codecs; the actual position
    /// after seeking may be at the nearest keyframe.
    fn seek(&mut self, position: Timestamp) -> MediaResult<()>;

    /// Get the current playback position.
    fn position(&self) -> Timestamp;

    /// Check if end of stream has been reached.
    fn is_eos(&self) -> bool;

    /// Reset the source to the beginning.
    fn reset(&mut self) -> MediaResult<()> {
        self.seek(Timestamp::ZERO)
    }

    /// Get the selected video stream index.
    fn video_stream_index(&self) -> Option<usize> {
        self.media_info().video_stream().map(|s| s.index)
    }

    /// Get the selected audio stream index.
    fn audio_stream_index(&self) -> Option<usize> {
        self.media_info().audio_stream().map(|s| s.index)
    }

    /// Select a specific video stream by index.
    fn select_video_stream(&mut self, _index: usize) -> MediaResult<()> {
        Err(MediaError::NotSupported("stream selection".to_string()))
    }

    /// Select a specific audio stream by index.
    fn select_audio_stream(&mut self, _index: usize) -> MediaResult<()> {
        Err(MediaError::NotSupported("stream selection".to_string()))
    }
}

/// A media source factory for creating sources from paths/URLs.
pub trait MediaSourceFactory {
    /// Check if this factory can handle the given path/URL.
    fn can_handle(&self, path: &str) -> bool;

    /// Open a media source from a file path.
    fn open_file(&self, path: &Path) -> MediaResult<Box<dyn MediaSource>>;

    /// Open a media source from a URL.
    fn open_url(&self, url: &str) -> MediaResult<Box<dyn MediaSource>>;

    /// Get a description of this factory (for logging).
    fn description(&self) -> &str;
}

/// A simple media source that yields a fixed sequence of frames.
///
/// Useful for testing and creating animations.
pub struct TestSource {
    info: MediaInfo,
    frames: Vec<VideoFrame>,
    current_frame: usize,
    frame_duration: Timestamp,
    position: Timestamp,
}

impl TestSource {
    /// Create a new test source with the given frames.
    pub fn new(frames: Vec<VideoFrame>, frame_rate: f64) -> Self {
        let frame_duration = Timestamp::from_secs(1.0 / frame_rate);
        let total_duration =
            Timestamp::from_micros((frames.len() as i64) * frame_duration.as_micros());

        let (width, height) = frames
            .first()
            .map(|f| (f.width, f.height))
            .unwrap_or((0, 0));

        let info = MediaInfo {
            format: "test".to_string(),
            duration: Some(total_duration),
            bitrate: None,
            streams: vec![StreamInfo::Video(VideoStreamInfo {
                index: 0,
                codec: "raw".to_string(),
                width,
                height,
                frame_rate,
                pixel_format: Some("rgba".to_string()),
                bit_depth: Some(8),
                color_space: None,
                frame_count: Some(frames.len() as u64),
                duration: Some(total_duration),
                bitrate: None,
            })],
            metadata: Vec::new(),
            seekable: true,
            is_live: false,
        };

        Self {
            info,
            frames,
            current_frame: 0,
            frame_duration,
            position: Timestamp::ZERO,
        }
    }

    /// Create a test source with solid color frames.
    pub fn solid_color(
        width: u32,
        height: u32,
        color: [u8; 4],
        frame_count: usize,
        frame_rate: f64,
    ) -> Self {
        let pixels: Vec<u8> = (0..(width * height))
            .flat_map(|_| color.iter().copied())
            .collect();

        let frames: Vec<VideoFrame> = (0..frame_count)
            .map(|i| {
                let pts = Timestamp::from_secs(i as f64 / frame_rate);
                VideoFrame::from_rgba_with_pts(
                    pixels.clone(),
                    width,
                    height,
                    pts,
                    Some(Timestamp::from_secs(1.0 / frame_rate)),
                )
            })
            .collect();

        Self::new(frames, frame_rate)
    }

    /// Create a test source with a color gradient animation.
    pub fn gradient(width: u32, height: u32, frame_count: usize, frame_rate: f64) -> Self {
        let frames: Vec<VideoFrame> = (0..frame_count)
            .map(|i| {
                let progress = i as f32 / frame_count as f32;
                let pixels: Vec<u8> = (0..height)
                    .flat_map(|y| {
                        (0..width).flat_map(move |x| {
                            let r = ((x as f32 / width as f32) * 255.0) as u8;
                            let g = ((y as f32 / height as f32) * 255.0) as u8;
                            let b = (progress * 255.0) as u8;
                            [r, g, b, 255u8]
                        })
                    })
                    .collect();

                let pts = Timestamp::from_secs(i as f64 / frame_rate);
                VideoFrame::from_rgba_with_pts(
                    pixels,
                    width,
                    height,
                    pts,
                    Some(Timestamp::from_secs(1.0 / frame_rate)),
                )
            })
            .collect();

        Self::new(frames, frame_rate)
    }
}

impl MediaSource for TestSource {
    fn media_info(&self) -> &MediaInfo {
        &self.info
    }

    fn read_video_frame(&mut self) -> MediaResult<Option<VideoFrame>> {
        if self.current_frame >= self.frames.len() {
            return Ok(None);
        }

        let frame = self.frames[self.current_frame].clone();
        self.current_frame += 1;
        self.position = self.position + self.frame_duration;

        Ok(Some(frame))
    }

    fn read_audio_samples(&mut self, _buffer: &mut [f32]) -> MediaResult<usize> {
        Ok(0)
    }

    fn seek(&mut self, position: Timestamp) -> MediaResult<()> {
        let frame_index = (position.as_secs()
            * self
                .info
                .video_stream()
                .map(|s| s.frame_rate)
                .unwrap_or(30.0)) as usize;

        self.current_frame = frame_index.min(self.frames.len());
        self.position =
            Timestamp::from_micros((self.current_frame as i64) * self.frame_duration.as_micros());

        Ok(())
    }

    fn position(&self) -> Timestamp {
        self.position
    }

    fn is_eos(&self) -> bool {
        self.current_frame >= self.frames.len()
    }
}

/// An empty media source that produces no frames.
///
/// Useful as a placeholder or for audio-only scenarios.
pub struct EmptySource {
    info: MediaInfo,
}

impl EmptySource {
    /// Create a new empty source.
    pub fn new() -> Self {
        Self {
            info: MediaInfo::empty(),
        }
    }
}

impl Default for EmptySource {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaSource for EmptySource {
    fn media_info(&self) -> &MediaInfo {
        &self.info
    }

    fn read_video_frame(&mut self) -> MediaResult<Option<VideoFrame>> {
        Ok(None)
    }

    fn read_audio_samples(&mut self, _buffer: &mut [f32]) -> MediaResult<usize> {
        Ok(0)
    }

    fn seek(&mut self, _position: Timestamp) -> MediaResult<()> {
        Ok(())
    }

    fn position(&self) -> Timestamp {
        Timestamp::ZERO
    }

    fn is_eos(&self) -> bool {
        true
    }
}
