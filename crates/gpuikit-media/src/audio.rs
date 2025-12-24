//! Audio playback support using rodio.
//!
//! This module provides audio playback capabilities for media content.
//! It handles:
//!
//! - Audio device initialization and management
//! - Playback controls (play, pause, stop, seek)
//! - Volume and mute control
//! - Audio stream management

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context as _, Result};
use crossbeam_channel::Sender;
use gpui::{App, Context, EventEmitter, Task};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

use crate::{PlaybackSpeed, Timestamp, Volume};

/// Global audio output state.
static AUDIO_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the audio subsystem.
///
/// This sets up the default audio output device. Should be called
/// during application initialization.
pub fn init(_cx: &mut App) {
    if AUDIO_INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    log::info!("Audio subsystem initialized");
}

/// Audio output handle for playing audio streams.
pub struct AudioOutput {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioOutput {
    /// Create a new audio output using the default device.
    pub fn new() -> Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().context("Failed to open default audio output device")?;

        Ok(Self {
            _stream: stream,
            handle,
        })
    }

    /// Get a handle for creating audio sinks.
    pub fn handle(&self) -> &OutputStreamHandle {
        &self.handle
    }

    /// Create a new sink for audio playback.
    pub fn create_sink(&self) -> Result<Sink> {
        Sink::try_new(&self.handle).context("Failed to create audio sink")
    }
}

/// Events emitted by the audio player.
#[derive(Debug, Clone)]
pub enum AudioPlayerEvent {
    /// Playback started.
    Playing,
    /// Playback paused.
    Paused,
    /// Playback stopped.
    Stopped,
    /// Volume changed.
    VolumeChanged(Volume),
    /// Playback position changed (e.g., from seeking).
    PositionChanged(Timestamp),
    /// An error occurred.
    Error(String),
    /// End of audio stream reached.
    Ended,
}

/// Commands sent to the audio playback thread.
#[derive(Debug)]
#[allow(dead_code)]
enum AudioCommand {
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(Timestamp),
    SetSpeed(f32),
    Shutdown,
}

/// Audio player entity for GPUI.
///
/// Manages audio playback state and provides controls for
/// play, pause, volume, and seeking.
pub struct AudioPlayer {
    /// Current playback state.
    state: AudioPlaybackState,

    /// Current volume (0.0 to 1.0+).
    volume: Volume,

    /// Whether audio is muted.
    muted: bool,

    /// Previous volume before muting.
    pre_mute_volume: Volume,

    /// Current playback speed.
    speed: PlaybackSpeed,

    /// Channel to send commands to the audio thread.
    command_tx: Option<Sender<AudioCommand>>,

    /// Current playback position.
    position: Timestamp,

    /// Total duration of the audio.
    duration: Option<Timestamp>,

    /// Background task managing audio playback.
    _playback_task: Option<Task<()>>,
}

/// Current state of audio playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlaybackState {
    /// No audio loaded or stopped.
    Stopped,
    /// Audio is playing.
    Playing,
    /// Audio is paused.
    Paused,
    /// Loading audio source.
    Loading,
}

impl EventEmitter<AudioPlayerEvent> for AudioPlayer {}

impl AudioPlayer {
    /// Create a new audio player.
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            state: AudioPlaybackState::Stopped,
            volume: Volume::DEFAULT,
            muted: false,
            pre_mute_volume: Volume::DEFAULT,
            speed: PlaybackSpeed::NORMAL,
            command_tx: None,
            position: Timestamp::ZERO,
            duration: None,
            _playback_task: None,
        }
    }

    /// Get the current playback state.
    pub fn state(&self) -> AudioPlaybackState {
        self.state
    }

    /// Check if audio is currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == AudioPlaybackState::Playing
    }

    /// Check if audio is paused.
    pub fn is_paused(&self) -> bool {
        self.state == AudioPlaybackState::Paused
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

    /// Get the current playback position.
    pub fn position(&self) -> Timestamp {
        self.position
    }

    /// Get the total duration if known.
    pub fn duration(&self) -> Option<Timestamp> {
        self.duration
    }

    /// Start or resume playback.
    pub fn play(&mut self, cx: &mut Context<Self>) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::Play);
            self.state = AudioPlaybackState::Playing;
            cx.emit(AudioPlayerEvent::Playing);
            cx.notify();
        }
    }

    /// Pause playback.
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::Pause);
            self.state = AudioPlaybackState::Paused;
            cx.emit(AudioPlayerEvent::Paused);
            cx.notify();
        }
    }

    /// Toggle between play and pause.
    pub fn toggle_playback(&mut self, cx: &mut Context<Self>) {
        match self.state {
            AudioPlaybackState::Playing => self.pause(cx),
            AudioPlaybackState::Paused => self.play(cx),
            _ => {}
        }
    }

    /// Stop playback and reset position.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::Stop);
        }
        self.state = AudioPlaybackState::Stopped;
        self.position = Timestamp::ZERO;
        cx.emit(AudioPlayerEvent::Stopped);
        cx.notify();
    }

    /// Set the volume level.
    pub fn set_volume(&mut self, volume: Volume, cx: &mut Context<Self>) {
        self.volume = volume;
        self.muted = false;

        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::SetVolume(volume.0));
        }

        cx.emit(AudioPlayerEvent::VolumeChanged(volume));
        cx.notify();
    }

    /// Mute audio (preserves volume setting).
    pub fn mute(&mut self, cx: &mut Context<Self>) {
        if !self.muted {
            self.pre_mute_volume = self.volume;
            self.muted = true;

            if let Some(tx) = &self.command_tx {
                let _ = tx.send(AudioCommand::SetVolume(0.0));
            }

            cx.emit(AudioPlayerEvent::VolumeChanged(Volume::MUTED));
            cx.notify();
        }
    }

    /// Unmute audio (restores previous volume).
    pub fn unmute(&mut self, cx: &mut Context<Self>) {
        if self.muted {
            self.muted = false;

            if let Some(tx) = &self.command_tx {
                let _ = tx.send(AudioCommand::SetVolume(self.pre_mute_volume.0));
            }

            cx.emit(AudioPlayerEvent::VolumeChanged(self.pre_mute_volume));
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

        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::SetSpeed(speed.0));
        }

        cx.notify();
    }

    /// Seek to a specific position.
    pub fn seek(&mut self, position: Timestamp, cx: &mut Context<Self>) {
        self.position = position;

        if let Some(tx) = &self.command_tx {
            let _ = tx.send(AudioCommand::Seek(position));
        }

        cx.emit(AudioPlayerEvent::PositionChanged(position));
        cx.notify();
    }

    /// Update the current position (called from playback thread).
    #[allow(dead_code)]
    pub(crate) fn update_position(&mut self, position: Timestamp, cx: &mut Context<Self>) {
        self.position = position;
        cx.notify();
    }

    /// Set the duration (called when loading audio).
    #[allow(dead_code)]
    pub(crate) fn set_duration(&mut self, duration: Timestamp, cx: &mut Context<Self>) {
        self.duration = Some(duration);
        cx.notify();
    }

    /// Handle playback ending.
    #[allow(dead_code)]
    pub(crate) fn on_ended(&mut self, cx: &mut Context<Self>) {
        self.state = AudioPlaybackState::Stopped;
        cx.emit(AudioPlayerEvent::Ended);
        cx.notify();
    }

    /// Handle an error.
    #[allow(dead_code)]
    pub(crate) fn on_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.state = AudioPlaybackState::Stopped;
        cx.emit(AudioPlayerEvent::Error(error));
        cx.notify();
    }
}

/// An active audio stream that can be attached to an audio player.
///
/// This wraps a rodio source and provides position tracking.
pub enum AudioStream {
    /// Audio output stream being played.
    Output {
        /// Cleanup callback when dropped.
        _drop: Box<dyn Send>,
    },
    /// Input stream for capture.
    Input {
        /// Cleanup callback when dropped.
        _drop: Box<dyn Send>,
    },
}

impl AudioStream {
    /// Create a dummy/empty audio stream.
    pub fn empty() -> Self {
        Self::Output {
            _drop: Box::new(()),
        }
    }
}

/// Audio samples buffer for decoded audio data.
pub struct AudioBuffer {
    /// Interleaved samples (f32).
    samples: Vec<f32>,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Number of channels.
    channels: u16,
    /// Current read position.
    position: usize,
}

impl AudioBuffer {
    /// Create a new audio buffer.
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            position: 0,
        }
    }

    /// Create an empty audio buffer.
    pub fn empty(sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            channels,
            position: 0,
        }
    }

    /// Get the duration of the buffer.
    pub fn duration(&self) -> Duration {
        let total_samples = self.samples.len() / self.channels as usize;
        Duration::from_secs_f64(total_samples as f64 / self.sample_rate as f64)
    }

    /// Reset playback position to the beginning.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Seek to a specific position.
    pub fn seek(&mut self, position: Duration) {
        let sample_offset =
            (position.as_secs_f64() * self.sample_rate as f64) as usize * self.channels as usize;
        self.position = sample_offset.min(self.samples.len());
    }
}

impl Iterator for AudioBuffer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for AudioBuffer {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration())
    }
}

/// Helper to create a simple tone for testing.
pub fn create_test_tone(frequency: f32, duration: Duration, sample_rate: u32) -> AudioBuffer {
    let num_samples = (duration.as_secs_f64() * sample_rate as f64) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
        samples.push(sample);
        samples.push(sample);
    }

    AudioBuffer::new(samples, sample_rate, 2)
}

/// Get the default audio output device info.
pub fn default_output_device() -> Result<String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("No audio output device available")?;

    Ok(device.name().unwrap_or_else(|_| "Unknown".to_string()))
}

/// Get the default audio input device info.
pub fn default_input_device() -> Result<String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No audio input device available")?;

    Ok(device.name().unwrap_or_else(|_| "Unknown".to_string()))
}

/// List available audio output devices.
pub fn list_output_devices() -> Result<Vec<String>> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let devices: Vec<String> = host
        .output_devices()
        .context("Failed to enumerate output devices")?
        .filter_map(|d| d.name().ok())
        .collect();

    Ok(devices)
}

/// List available audio input devices.
pub fn list_input_devices() -> Result<Vec<String>> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let devices: Vec<String> = host
        .input_devices()
        .context("Failed to enumerate input devices")?
        .filter_map(|d| d.name().ok())
        .collect();

    Ok(devices)
}
