//! Video player entity for GPUI applications.
//!
//! The [`VideoPlayer`] manages video playback, including:
//!
//! - Frame decoding and buffering
//! - Playback state (play, pause, stop, seek)
//! - A/V synchronization
//! - Integration with audio playback
//!
//! # Example
//!
//! ```ignore
//! use gpuikit_media::{VideoPlayer, VideoPlayerEvent};
//!
//! fn setup_player(window: &mut Window, cx: &mut Context<MyView>) {
//!     let player = cx.new(|cx| VideoPlayer::new(cx));
//!
//!     // Subscribe to events
//!     cx.subscribe(&player, |this, _player, event, cx| {
//!         match event {
//!             VideoPlayerEvent::FrameReady => cx.notify(),
//!             VideoPlayerEvent::PlaybackEnded => { /* handle end */ }
//!             VideoPlayerEvent::Error(msg) => log::error!("Player error: {}", msg),
//!             _ => {}
//!         }
//!     }).detach();
//!
//!     this.player = Some(player);
//! }
//! ```

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{Context, Entity, EventEmitter, Task};
use parking_lot::Mutex;

use crate::audio::AudioPlayer;
use crate::frame::VideoFrame;
use crate::source::{MediaInfo, MediaSource};
use crate::{MediaDuration, PlaybackSpeed, Timestamp, Volume};

/// Events emitted by the video player.
#[derive(Debug, Clone)]
pub enum VideoPlayerEvent {
    /// A new frame is ready for display.
    FrameReady,
    /// Playback state changed.
    StateChanged(PlaybackState),
    /// Playback started.
    Playing,
    /// Playback paused.
    Paused,
    /// Playback stopped.
    Stopped,
    /// Seeking started.
    SeekStarted(Timestamp),
    /// Seeking completed.
    SeekCompleted(Timestamp),
    /// Playback position updated (emitted periodically).
    PositionChanged(Timestamp),
    /// Duration is now known.
    DurationChanged(Timestamp),
    /// Volume changed.
    VolumeChanged(Volume),
    /// End of media reached.
    PlaybackEnded,
    /// Buffering started.
    BufferingStarted,
    /// Buffering progress (0-100).
    BufferingProgress(u8),
    /// Buffering completed.
    BufferingComplete,
    /// An error occurred.
    Error(String),
    /// Media source loaded successfully.
    MediaLoaded(MediaInfo),
}

/// Current playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// No media loaded.
    #[default]
    Empty,
    /// Media is loading.
    Loading,
    /// Ready to play (paused at start).
    Ready,
    /// Currently playing.
    Playing,
    /// Paused.
    Paused,
    /// Stopped (position reset to beginning).
    Stopped,
    /// Buffering (waiting for data).
    Buffering,
    /// Seeking to a new position.
    Seeking,
    /// Playback ended.
    Ended,
    /// Error state.
    Error,
}

impl PlaybackState {
    /// Check if playback is active (playing or buffering).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Playing | Self::Buffering)
    }

    /// Check if the player can accept play commands.
    pub fn can_play(&self) -> bool {
        matches!(
            self,
            Self::Ready | Self::Paused | Self::Stopped | Self::Ended
        )
    }

    /// Check if the player can accept pause commands.
    pub fn can_pause(&self) -> bool {
        matches!(self, Self::Playing | Self::Buffering)
    }

    /// Check if the player has media loaded.
    pub fn has_media(&self) -> bool {
        !matches!(self, Self::Empty | Self::Loading | Self::Error)
    }
}

/// Configuration for the video player.
#[derive(Debug, Clone)]
pub struct VideoPlayerConfig {
    /// Maximum number of frames to buffer ahead.
    pub buffer_size: usize,
    /// Whether to loop playback.
    pub loop_playback: bool,
    /// Whether to auto-play when media is loaded.
    pub auto_play: bool,
    /// Initial volume.
    pub initial_volume: Volume,
    /// How often to emit position updates (in milliseconds).
    pub position_update_interval_ms: u64,
}

impl Default for VideoPlayerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 3,
            loop_playback: false,
            auto_play: false,
            initial_volume: Volume::DEFAULT,
            position_update_interval_ms: 250,
        }
    }
}

/// The main video player entity.
///
/// Manages video decoding, frame buffering, and playback synchronization.
pub struct VideoPlayer {
    /// Current playback state.
    state: PlaybackState,

    /// Player configuration.
    config: VideoPlayerConfig,

    /// The current video frame to display.
    current_frame: Option<VideoFrame>,

    /// Buffered frames waiting to be displayed.
    frame_buffer: VecDeque<VideoFrame>,

    /// Information about the loaded media.
    media_info: Option<MediaInfo>,

    /// Current playback position.
    position: Timestamp,

    /// Total duration of the media.
    duration: Option<Timestamp>,

    /// Current volume level.
    volume: Volume,

    /// Whether audio is muted.
    muted: bool,

    /// Playback speed multiplier.
    speed: PlaybackSpeed,

    /// Time when playback started (for sync calculations).
    playback_start_time: Option<Instant>,

    /// Position when playback started.
    playback_start_position: Timestamp,

    /// Associated audio player (if media has audio).
    audio_player: Option<Entity<AudioPlayer>>,

    /// Background task for frame decoding.
    decode_task: Option<Task<()>>,

    /// Background task for playback timing.
    playback_task: Option<Task<()>>,

    /// Shared state for communication with background tasks.
    shared_state: Arc<SharedPlayerState>,

    /// Subscriptions to keep alive.
    _subscriptions: Vec<gpui::Subscription>,
}

/// Shared state between the player entity and background tasks.
struct SharedPlayerState {
    /// Flag to signal tasks to stop.
    should_stop: AtomicBool,
    /// Flag indicating playback is active.
    is_playing: AtomicBool,
    /// Pending seek position.
    pending_seek: Mutex<Option<Timestamp>>,
    /// Frame buffer for decoded frames.
    decoded_frames: Mutex<VecDeque<VideoFrame>>,
}

impl SharedPlayerState {
    fn new() -> Self {
        Self {
            should_stop: AtomicBool::new(false),
            is_playing: AtomicBool::new(false),
            pending_seek: Mutex::new(None),
            decoded_frames: Mutex::new(VecDeque::new()),
        }
    }

    fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    fn reset(&self) {
        self.should_stop.store(false, Ordering::SeqCst);
        self.is_playing.store(false, Ordering::SeqCst);
        *self.pending_seek.lock() = None;
        self.decoded_frames.lock().clear();
    }
}

impl EventEmitter<VideoPlayerEvent> for VideoPlayer {}

impl VideoPlayer {
    /// Create a new video player with default configuration.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::with_config(VideoPlayerConfig::default(), cx)
    }

    /// Create a new video player with custom configuration.
    pub fn with_config(config: VideoPlayerConfig, _cx: &mut Context<Self>) -> Self {
        Self {
            state: PlaybackState::Empty,
            config,
            current_frame: None,
            frame_buffer: VecDeque::new(),
            media_info: None,
            position: Timestamp::ZERO,
            duration: None,
            volume: Volume::DEFAULT,
            muted: false,
            speed: PlaybackSpeed::NORMAL,
            playback_start_time: None,
            playback_start_position: Timestamp::ZERO,
            audio_player: None,
            decode_task: None,
            playback_task: None,
            shared_state: Arc::new(SharedPlayerState::new()),
            _subscriptions: Vec::new(),
        }
    }

    /// Get the current playback state.
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Check if the player is currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Check if the player is paused.
    pub fn is_paused(&self) -> bool {
        self.state == PlaybackState::Paused
    }

    /// Check if media is loaded.
    pub fn has_media(&self) -> bool {
        self.state.has_media()
    }

    /// Get the current video frame for rendering.
    pub fn current_frame(&self) -> Option<&VideoFrame> {
        self.current_frame.as_ref()
    }

    /// Get media information if available.
    pub fn media_info(&self) -> Option<&MediaInfo> {
        self.media_info.as_ref()
    }

    /// Get the current playback position.
    pub fn position(&self) -> Timestamp {
        self.position
    }

    /// Get the total duration if known.
    pub fn duration(&self) -> Option<Timestamp> {
        self.duration
    }

    /// Get duration information for UI display.
    pub fn duration_info(&self) -> MediaDuration {
        MediaDuration {
            total: self.duration.unwrap_or(Timestamp::ZERO),
            current: self.position,
        }
    }

    /// Get the current volume.
    pub fn volume(&self) -> Volume {
        if self.muted {
            Volume::MUTED
        } else {
            self.volume
        }
    }

    /// Check if audio is muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Get the current playback speed.
    pub fn speed(&self) -> PlaybackSpeed {
        self.speed
    }

    /// Get the configuration.
    pub fn config(&self) -> &VideoPlayerConfig {
        &self.config
    }

    /// Update the configuration.
    pub fn set_config(&mut self, config: VideoPlayerConfig) {
        self.config = config;
    }

    /// Load media from a source.
    ///
    /// This will stop any current playback and load the new media.
    pub fn load_source(&mut self, source: Box<dyn MediaSource>, cx: &mut Context<Self>) {
        self.stop_internal();

        let info = source.media_info().clone();
        self.media_info = Some(info.clone());
        self.duration = info.duration;
        self.position = Timestamp::ZERO;
        self.state = PlaybackState::Ready;

        if let Some(duration) = info.duration {
            cx.emit(VideoPlayerEvent::DurationChanged(duration));
        }

        cx.emit(VideoPlayerEvent::MediaLoaded(info));
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Ready));

        self.start_decode_task(source, cx);

        if self.config.auto_play {
            self.play(cx);
        }

        cx.notify();
    }

    /// Load a frame directly (for simple use cases without a source).
    pub fn load_frame(&mut self, frame: VideoFrame, cx: &mut Context<Self>) {
        self.stop_internal();
        self.current_frame = Some(frame);
        self.state = PlaybackState::Ready;

        cx.emit(VideoPlayerEvent::FrameReady);
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Ready));
        cx.notify();
    }

    /// Start or resume playback.
    pub fn play(&mut self, cx: &mut Context<Self>) {
        if !self.state.can_play() {
            return;
        }

        self.playback_start_time = Some(Instant::now());
        self.playback_start_position = self.position;
        self.shared_state.is_playing.store(true, Ordering::SeqCst);

        self.state = PlaybackState::Playing;
        self.start_playback_task(cx);

        cx.emit(VideoPlayerEvent::Playing);
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Playing));
        cx.notify();
    }

    /// Pause playback.
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        if !self.state.can_pause() {
            return;
        }

        self.update_position_from_clock();
        self.playback_start_time = None;
        self.shared_state.is_playing.store(false, Ordering::SeqCst);

        self.state = PlaybackState::Paused;
        self.stop_playback_task();

        cx.emit(VideoPlayerEvent::Paused);
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Paused));
        cx.notify();
    }

    /// Toggle between play and pause.
    pub fn toggle_playback(&mut self, cx: &mut Context<Self>) {
        match self.state {
            PlaybackState::Playing => self.pause(cx),
            PlaybackState::Paused | PlaybackState::Ready | PlaybackState::Stopped => self.play(cx),
            PlaybackState::Ended if self.config.loop_playback => {
                self.seek(Timestamp::ZERO, cx);
                self.play(cx);
            }
            PlaybackState::Ended => {
                self.seek(Timestamp::ZERO, cx);
                self.play(cx);
            }
            _ => {}
        }
    }

    /// Stop playback and reset to the beginning.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if !self.state.has_media() {
            return;
        }

        self.stop_playback_task();
        self.shared_state.is_playing.store(false, Ordering::SeqCst);
        self.playback_start_time = None;
        self.frame_buffer.clear();

        self.position = Timestamp::ZERO;
        self.playback_start_position = Timestamp::ZERO;
        *self.shared_state.pending_seek.lock() = Some(Timestamp::ZERO);
        self.shared_state.decoded_frames.lock().clear();

        self.state = PlaybackState::Stopped;

        cx.emit(VideoPlayerEvent::Stopped);
        cx.emit(VideoPlayerEvent::StateChanged(self.state));
        cx.emit(VideoPlayerEvent::PositionChanged(Timestamp::ZERO));
        cx.notify();
    }

    /// Internal stop - fully stops decode task (used when loading new media).
    fn stop_internal(&mut self) {
        self.shared_state.stop();
        self.stop_playback_task();

        self.position = Timestamp::ZERO;
        self.playback_start_time = None;
        self.frame_buffer.clear();
    }

    /// Seek to a specific position.
    pub fn seek(&mut self, position: Timestamp, cx: &mut Context<Self>) {
        let was_playing = self.state == PlaybackState::Playing;

        let clamped_position = if let Some(duration) = self.duration {
            Timestamp::from_micros(position.as_micros().clamp(0, duration.as_micros()))
        } else {
            position
        };

        self.state = PlaybackState::Seeking;
        self.position = clamped_position;
        self.playback_start_position = clamped_position;
        self.playback_start_time = if was_playing {
            Some(Instant::now())
        } else {
            None
        };

        *self.shared_state.pending_seek.lock() = Some(clamped_position);
        self.frame_buffer.clear();

        cx.emit(VideoPlayerEvent::SeekStarted(clamped_position));
        cx.emit(VideoPlayerEvent::PositionChanged(clamped_position));

        self.state = if was_playing {
            PlaybackState::Playing
        } else {
            PlaybackState::Paused
        };

        cx.emit(VideoPlayerEvent::SeekCompleted(clamped_position));
        cx.emit(VideoPlayerEvent::StateChanged(self.state));
        cx.notify();
    }

    /// Seek forward by the given duration.
    pub fn seek_forward(&mut self, duration: Duration, cx: &mut Context<Self>) {
        let new_position = self.position + Timestamp::from(duration);
        self.seek(new_position, cx);
    }

    /// Seek backward by the given duration.
    pub fn seek_backward(&mut self, duration: Duration, cx: &mut Context<Self>) {
        let new_position = Timestamp::from_micros(
            (self.position.as_micros() - duration.as_micros() as i64).max(0),
        );
        self.seek(new_position, cx);
    }

    /// Seek to a percentage of the total duration (0.0 to 1.0).
    pub fn seek_percentage(&mut self, percentage: f64, cx: &mut Context<Self>) {
        if let Some(duration) = self.duration {
            let position = Timestamp::from_micros(
                (duration.as_micros() as f64 * percentage.clamp(0.0, 1.0)) as i64,
            );
            self.seek(position, cx);
        }
    }

    /// Set the volume level.
    pub fn set_volume(&mut self, volume: Volume, cx: &mut Context<Self>) {
        self.volume = volume;
        self.muted = false;

        if let Some(audio) = &self.audio_player {
            audio.update(cx, |audio, cx| {
                audio.set_volume(volume, cx);
            });
        }

        cx.emit(VideoPlayerEvent::VolumeChanged(volume));
        cx.notify();
    }

    /// Mute audio (preserves volume setting).
    pub fn mute(&mut self, cx: &mut Context<Self>) {
        if !self.muted {
            self.muted = true;

            if let Some(audio) = &self.audio_player {
                audio.update(cx, |audio, cx| {
                    audio.mute(cx);
                });
            }

            cx.emit(VideoPlayerEvent::VolumeChanged(Volume::MUTED));
            cx.notify();
        }
    }

    /// Unmute audio (restores previous volume).
    pub fn unmute(&mut self, cx: &mut Context<Self>) {
        if self.muted {
            self.muted = false;

            if let Some(audio) = &self.audio_player {
                audio.update(cx, |audio, cx| {
                    audio.unmute(cx);
                });
            }

            cx.emit(VideoPlayerEvent::VolumeChanged(self.volume));
            cx.notify();
        }
    }

    /// Toggle mute state.
    pub fn toggle_mute(&mut self, cx: &mut Context<Self>) {
        if self.muted {
            self.unmute(cx);
        } else {
            self.mute(cx);
        }
    }

    /// Set playback speed.
    pub fn set_speed(&mut self, speed: PlaybackSpeed, cx: &mut Context<Self>) {
        self.speed = speed;

        if self.playback_start_time.is_some() {
            self.update_position_from_clock();
            self.playback_start_time = Some(Instant::now());
            self.playback_start_position = self.position;
        }

        if let Some(audio) = &self.audio_player {
            audio.update(cx, |audio, cx| {
                audio.set_speed(speed, cx);
            });
        }

        cx.notify();
    }

    /// Enable or disable looping.
    pub fn set_loop(&mut self, loop_playback: bool) {
        self.config.loop_playback = loop_playback;
    }

    /// Update position based on the playback clock.
    fn update_position_from_clock(&mut self) {
        if let Some(start_time) = self.playback_start_time {
            let elapsed = start_time.elapsed();
            let elapsed_scaled =
                Duration::from_secs_f64(elapsed.as_secs_f64() * self.speed.0 as f64);
            self.position = self.playback_start_position + Timestamp::from(elapsed_scaled);

            if let Some(duration) = self.duration {
                if self.position.as_micros() >= duration.as_micros() {
                    self.position = duration;
                }
            }
        }
    }

    /// Start the frame decode background task.
    fn start_decode_task(&mut self, mut source: Box<dyn MediaSource>, cx: &mut Context<Self>) {
        let shared_state = self.shared_state.clone();
        let buffer_size = self.config.buffer_size;

        shared_state.reset();

        self.decode_task = Some(cx.spawn(async move |this, cx| loop {
            if shared_state.should_stop.load(Ordering::SeqCst) {
                break;
            }

            if let Some(seek_pos) = shared_state.pending_seek.lock().take() {
                if let Err(e) = source.seek(seek_pos) {
                    let error_msg = e.to_string();
                    this.update(cx, |this, cx| {
                        this.on_error(error_msg, cx);
                    })
                    .ok();
                    break;
                }
            }

            let buffer_len = shared_state.decoded_frames.lock().len();
            if buffer_len >= buffer_size {
                cx.background_executor()
                    .timer(Duration::from_millis(10))
                    .await;
                continue;
            }

            match source.read_video_frame() {
                Ok(Some(frame)) => {
                    shared_state.decoded_frames.lock().push_back(frame);
                }
                Ok(None) => {
                    this.update(cx, |this, cx| {
                        this.on_end_of_stream(cx);
                    })
                    .ok();
                    break;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    this.update(cx, |this, cx| {
                        this.on_error(error_msg, cx);
                    })
                    .ok();
                    break;
                }
            }
        }));
    }

    /// Start the playback timing task.
    fn start_playback_task(&mut self, cx: &mut Context<Self>) {
        self.stop_playback_task();

        let shared_state = self.shared_state.clone();
        let _update_interval = Duration::from_millis(self.config.position_update_interval_ms);

        self.playback_task = Some(cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;

            if shared_state.should_stop.load(Ordering::SeqCst) {
                break;
            }

            if !shared_state.is_playing.load(Ordering::SeqCst) {
                break;
            }

            let frame = shared_state.decoded_frames.lock().pop_front();

            let should_emit_position = this
                .update(cx, |this, cx| {
                    this.update_position_from_clock();

                    if let Some(frame) = frame {
                        this.current_frame = Some(frame);
                        cx.emit(VideoPlayerEvent::FrameReady);
                    }

                    if let Some(duration) = this.duration {
                        if this.position.as_micros() >= duration.as_micros() {
                            this.on_playback_complete(cx);
                            return false;
                        }
                    }

                    cx.notify();
                    true
                })
                .unwrap_or(false);

            if !should_emit_position {
                break;
            }
        }));
    }

    /// Stop the playback timing task.
    fn stop_playback_task(&mut self) {
        self.playback_task = None;
    }

    /// Handle end of stream.
    fn on_end_of_stream(&mut self, cx: &mut Context<Self>) {
        if self.config.loop_playback {
            self.position = Timestamp::ZERO;
            self.playback_start_position = Timestamp::ZERO;
            self.playback_start_time = Some(Instant::now());
            *self.shared_state.pending_seek.lock() = Some(Timestamp::ZERO);
        } else {
            self.on_playback_complete(cx);
        }
    }

    /// Handle playback completion.
    fn on_playback_complete(&mut self, cx: &mut Context<Self>) {
        self.state = PlaybackState::Ended;
        self.shared_state.is_playing.store(false, Ordering::SeqCst);
        self.playback_start_time = None;

        cx.emit(VideoPlayerEvent::PlaybackEnded);
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Ended));
        cx.notify();
    }

    /// Handle an error.
    fn on_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.state = PlaybackState::Error;
        self.shared_state.stop();

        cx.emit(VideoPlayerEvent::Error(error));
        cx.emit(VideoPlayerEvent::StateChanged(PlaybackState::Error));
        cx.notify();
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.shared_state.stop();
    }
}

/// Builder for creating video players with custom settings.
pub struct VideoPlayerBuilder {
    config: VideoPlayerConfig,
}

impl VideoPlayerBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            config: VideoPlayerConfig::default(),
        }
    }

    /// Set the frame buffer size.
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Enable or disable looping.
    pub fn loop_playback(mut self, enabled: bool) -> Self {
        self.config.loop_playback = enabled;
        self
    }

    /// Enable or disable auto-play.
    pub fn auto_play(mut self, enabled: bool) -> Self {
        self.config.auto_play = enabled;
        self
    }

    /// Set the initial volume.
    pub fn initial_volume(mut self, volume: Volume) -> Self {
        self.config.initial_volume = volume;
        self
    }

    /// Set the position update interval.
    pub fn position_update_interval(mut self, interval: Duration) -> Self {
        self.config.position_update_interval_ms = interval.as_millis() as u64;
        self
    }

    /// Build the video player.
    pub fn build(self, cx: &mut Context<VideoPlayer>) -> VideoPlayer {
        VideoPlayer::with_config(self.config, cx)
    }
}

impl Default for VideoPlayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
