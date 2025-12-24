//! gpuikit-media - Video and audio playback support for gpuikit
//!
//! This crate provides infrastructure for playing video and audio content
//! in GPUI applications. It handles:
//!
//! - Cross-platform video frame representation
//! - Video player entity with playback controls
//! - Audio playback through rodio
//! - A/V synchronization
//!
//! # Architecture
//!
//! The media system is built around these core concepts:
//!
//! - [`VideoFrame`]: A platform-optimized video frame (CVPixelBuffer on macOS, RGBA elsewhere)
//! - [`VideoPlayer`]: A GPUI entity that manages video playback state
//! - [`AudioPlayer`]: Audio playback through the system audio device
//! - [`MediaSource`]: Trait for implementing video/audio decoders
//!
//! # Example
//!
//! ```ignore
//! use gpuikit_media::{VideoPlayer, VideoPlayerEvent};
//! use gpui::{Entity, Window, Context};
//!
//! fn create_player(window: &mut Window, cx: &mut Context<MyView>) -> Entity<VideoPlayer> {
//!     let player = cx.new(|cx| VideoPlayer::new(cx));
//!
//!     // Subscribe to player events
//!     cx.subscribe(&player, |this, _player, event, cx| {
//!         match event {
//!             VideoPlayerEvent::FrameReady => cx.notify(),
//!             VideoPlayerEvent::PlaybackEnded => { /* handle end */ }
//!             _ => {}
//!         }
//!     }).detach();
//!
//!     player
//! }
//! ```

use std::time::Duration;

use gpui::App;

pub mod audio;
pub mod ffmpeg;
pub mod frame;
pub mod player;
pub mod source;
pub mod view;

pub use audio::{AudioPlayer, AudioPlayerEvent, AudioStream};
pub use ffmpeg::FfmpegSource;
pub use frame::VideoFrame;
pub use player::{PlaybackState, VideoPlayer, VideoPlayerConfig, VideoPlayerEvent};
pub use source::{FrameInfo, MediaInfo, MediaSource, MediaSourceEvent, TestSource};
pub use view::{video_player_view, video_view, VideoPlayerView, VideoView};

/// Initialize the media subsystem.
///
/// This should be called during application setup. It initializes
/// audio output devices and any platform-specific media infrastructure.
pub fn init(cx: &mut App) {
    audio::init(cx);
}

/// Timestamp for media synchronization, in microseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// Zero timestamp.
    pub const ZERO: Self = Self(0);

    /// Create a timestamp from seconds.
    pub fn from_secs(secs: f64) -> Self {
        Self((secs * 1_000_000.0) as i64)
    }

    /// Create a timestamp from milliseconds.
    pub fn from_millis(millis: i64) -> Self {
        Self(millis * 1000)
    }

    /// Create a timestamp from microseconds.
    pub fn from_micros(micros: i64) -> Self {
        Self(micros)
    }

    /// Convert to seconds.
    pub fn as_secs(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    /// Convert to milliseconds.
    pub fn as_millis(&self) -> i64 {
        self.0 / 1000
    }

    /// Convert to microseconds.
    pub fn as_micros(&self) -> i64 {
        self.0
    }

    /// Convert to a Duration (only valid for non-negative timestamps).
    pub fn as_duration(&self) -> Option<Duration> {
        if self.0 >= 0 {
            Some(Duration::from_micros(self.0 as u64))
        } else {
            None
        }
    }
}

impl From<Duration> for Timestamp {
    fn from(duration: Duration) -> Self {
        Self(duration.as_micros() as i64)
    }
}

impl std::ops::Add for Timestamp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Media duration information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaDuration {
    /// Total duration of the media.
    pub total: Timestamp,
    /// Current playback position.
    pub current: Timestamp,
}

impl MediaDuration {
    /// Calculate progress as a value between 0.0 and 1.0.
    pub fn progress(&self) -> f64 {
        if self.total.0 == 0 {
            0.0
        } else {
            (self.current.0 as f64 / self.total.0 as f64).clamp(0.0, 1.0)
        }
    }
}

/// Volume level (0.0 to 1.0, can exceed 1.0 for amplification).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume(pub f32);

impl Volume {
    /// Muted volume.
    pub const MUTED: Self = Self(0.0);
    /// Default volume (100%).
    pub const DEFAULT: Self = Self(1.0);
    /// Maximum recommended volume.
    pub const MAX: Self = Self(2.0);

    /// Create a new volume level.
    pub fn new(level: f32) -> Self {
        Self(level.max(0.0))
    }

    /// Check if muted.
    pub fn is_muted(&self) -> bool {
        self.0 == 0.0
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Playback speed multiplier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed(pub f32);

impl PlaybackSpeed {
    /// Normal speed (1x).
    pub const NORMAL: Self = Self(1.0);
    /// Half speed (0.5x).
    pub const HALF: Self = Self(0.5);
    /// Double speed (2x).
    pub const DOUBLE: Self = Self(2.0);

    /// Create a new playback speed.
    pub fn new(speed: f32) -> Self {
        Self(speed.max(0.1).min(16.0))
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Errors that can occur during media operations.
#[derive(Debug, Clone)]
pub enum MediaError {
    /// Failed to open the media source.
    OpenFailed(String),
    /// Decoder error.
    DecodeFailed(String),
    /// Audio output error.
    AudioError(String),
    /// Seek failed.
    SeekFailed(String),
    /// End of stream reached.
    EndOfStream,
    /// Operation not supported.
    NotSupported(String),
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed(msg) => write!(f, "Failed to open media: {}", msg),
            Self::DecodeFailed(msg) => write!(f, "Decode error: {}", msg),
            Self::AudioError(msg) => write!(f, "Audio error: {}", msg),
            Self::SeekFailed(msg) => write!(f, "Seek failed: {}", msg),
            Self::EndOfStream => write!(f, "End of stream"),
            Self::NotSupported(msg) => write!(f, "Not supported: {}", msg),
        }
    }
}

impl std::error::Error for MediaError {}

/// Result type for media operations.
pub type MediaResult<T> = Result<T, MediaError>;
