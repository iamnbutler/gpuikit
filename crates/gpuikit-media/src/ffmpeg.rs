//! FFmpeg-based media source for video file decoding.
//!
//! This module provides [`FfmpegSource`], which uses FFmpeg (via ffmpeg-sidecar)
//! to decode video files. It implements the [`MediaSource`] trait.
//!
//! # Requirements
//!
//! FFmpeg must be available. The ffmpeg-sidecar crate will download it
//! automatically if not found.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit_media::ffmpeg::FfmpegSource;
//! use gpuikit_media::VideoPlayer;
//!
//! let source = FfmpegSource::open("/path/to/video.mp4")?;
//! player.load_source(Box::new(source), cx);
//! ```

use std::path::{Path, PathBuf};

use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::{FfmpegEvent, OutputVideoFrame};
use ffmpeg_sidecar::iter::FfmpegIterator;

use crate::frame::VideoFrame;
use crate::source::{AudioStreamInfo, MediaInfo, MediaSource, StreamInfo, VideoStreamInfo};
use crate::{MediaError, MediaResult, Timestamp};

/// A media source that decodes video files using FFmpeg.
///
/// This implementation spawns FFmpeg as a subprocess and reads decoded
/// frames through a pipe. It supports most video formats and codecs
/// that FFmpeg can handle.
pub struct FfmpegSource {
    path: PathBuf,
    info: MediaInfo,
    child: Option<FfmpegChild>,
    iter: Option<FfmpegIterator>,
    position: Timestamp,
    frame_duration: Timestamp,
    eos: bool,
    width: u32,
    height: u32,
}

impl FfmpegSource {
    /// Open a video file for decoding.
    ///
    /// This probes the file for metadata and prepares FFmpeg for decoding.
    pub fn open<P: AsRef<Path>>(path: P) -> MediaResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(MediaError::OpenFailed(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let (info, width, height) = probe_with_ffmpeg(&path)?;

        let frame_rate = info.video_stream().map(|v| v.frame_rate).unwrap_or(30.0);

        let frame_duration = Timestamp::from_secs(1.0 / frame_rate);

        let mut source = Self {
            path,
            info,
            child: None,
            iter: None,
            position: Timestamp::ZERO,
            frame_duration,
            eos: false,
            width,
            height,
        };

        source.start_decoding(Timestamp::ZERO)?;

        Ok(source)
    }

    fn start_decoding(&mut self, start_position: Timestamp) -> MediaResult<()> {
        self.stop_decoding();

        let path_str = self
            .path
            .to_str()
            .ok_or_else(|| MediaError::OpenFailed("Invalid path encoding".to_string()))?;

        let mut cmd = FfmpegCommand::new();

        if start_position.as_micros() > 0 {
            cmd.args(["-ss", &format!("{:.3}", start_position.as_secs())]);
        }

        let mut child = cmd
            .input(path_str)
            .args(["-vf", &format!("scale={}:{}", self.width, self.height)])
            .rawvideo()
            .spawn()
            .map_err(|e| MediaError::OpenFailed(format!("Failed to spawn FFmpeg: {}", e)))?;

        let iter = child.iter().map_err(|e| {
            MediaError::OpenFailed(format!("Failed to create FFmpeg iterator: {}", e))
        })?;

        self.child = Some(child);
        self.iter = Some(iter);
        self.position = start_position;
        self.eos = false;

        Ok(())
    }

    fn stop_decoding(&mut self) {
        self.iter = None;
        if let Some(mut child) = self.child.take() {
            child.kill().ok();
        }
    }

    fn convert_frame_to_rgba(&self, frame: &OutputVideoFrame) -> Vec<u8> {
        let pixel_count = (frame.width * frame.height) as usize;
        let mut rgba = vec![255u8; pixel_count * 4];

        match frame.pix_fmt.as_str() {
            "rgb24" => {
                for i in 0..pixel_count {
                    let src_offset = i * 3;
                    let dst_offset = i * 4;
                    if src_offset + 2 < frame.data.len() {
                        rgba[dst_offset] = frame.data[src_offset + 2];
                        rgba[dst_offset + 1] = frame.data[src_offset + 1];
                        rgba[dst_offset + 2] = frame.data[src_offset];
                        rgba[dst_offset + 3] = 255;
                    }
                }
            }
            "rgba" => {
                let copy_len = rgba.len().min(frame.data.len());
                rgba[..copy_len].copy_from_slice(&frame.data[..copy_len]);
            }
            "bgra" => {
                for i in 0..pixel_count {
                    let offset = i * 4;
                    if offset + 3 < frame.data.len() {
                        rgba[offset] = frame.data[offset + 2];
                        rgba[offset + 1] = frame.data[offset + 1];
                        rgba[offset + 2] = frame.data[offset];
                        rgba[offset + 3] = frame.data[offset + 3];
                    }
                }
            }
            _ => {
                for i in 0..pixel_count {
                    let src_offset = i * 3;
                    let dst_offset = i * 4;
                    if src_offset + 2 < frame.data.len() {
                        rgba[dst_offset] = frame.data[src_offset + 2];
                        rgba[dst_offset + 1] = frame.data[src_offset + 1];
                        rgba[dst_offset + 2] = frame.data[src_offset];
                        rgba[dst_offset + 3] = 255;
                    }
                }
            }
        }

        rgba
    }
}

impl MediaSource for FfmpegSource {
    fn media_info(&self) -> &MediaInfo {
        &self.info
    }

    fn read_video_frame(&mut self) -> MediaResult<Option<VideoFrame>> {
        if self.eos {
            return Ok(None);
        }

        let iter = match &mut self.iter {
            Some(it) => it,
            None => return Ok(None),
        };

        for event in iter {
            match event {
                FfmpegEvent::OutputFrame(frame) => {
                    let rgba_data = self.convert_frame_to_rgba(&frame);
                    let pts = self.position;
                    let duration = Some(self.frame_duration);

                    self.position = self.position + self.frame_duration;

                    let video_frame = VideoFrame::from_rgba_with_pts(
                        rgba_data,
                        frame.width,
                        frame.height,
                        pts,
                        duration,
                    );

                    return Ok(Some(video_frame));
                }
                FfmpegEvent::Done => {
                    self.eos = true;
                    return Ok(None);
                }
                FfmpegEvent::Error(error) => {
                    return Err(MediaError::DecodeFailed(error));
                }
                _ => {
                    continue;
                }
            }
        }

        self.eos = true;
        Ok(None)
    }

    fn read_audio_samples(&mut self, _buffer: &mut [f32]) -> MediaResult<usize> {
        Ok(0)
    }

    fn seek(&mut self, position: Timestamp) -> MediaResult<()> {
        self.start_decoding(position)
    }

    fn position(&self) -> Timestamp {
        self.position
    }

    fn is_eos(&self) -> bool {
        self.eos
    }

    fn reset(&mut self) -> MediaResult<()> {
        self.start_decoding(Timestamp::ZERO)
    }
}

impl Drop for FfmpegSource {
    fn drop(&mut self) {
        self.stop_decoding();
    }
}

/// Probe video metadata using FFmpeg.
/// We run FFmpeg briefly to extract stream information from its output.
fn probe_with_ffmpeg(path: &Path) -> MediaResult<(MediaInfo, u32, u32)> {
    let path_str = path
        .to_str()
        .ok_or_else(|| MediaError::OpenFailed("Invalid path encoding".to_string()))?;

    let mut child = FfmpegCommand::new()
        .args(["-i", path_str])
        .args(["-f", "null", "-"])
        .spawn()
        .map_err(|e| MediaError::OpenFailed(format!("Failed to spawn FFmpeg: {}", e)))?;

    let iter = child
        .iter()
        .map_err(|e| MediaError::OpenFailed(format!("Failed to iterate FFmpeg: {}", e)))?;

    let mut duration: Option<Timestamp> = None;
    let mut width: u32 = 640;
    let mut height: u32 = 480;
    let mut frame_rate: f64 = 30.0;
    let mut video_codec = String::from("unknown");
    let mut audio_codec = String::new();
    let mut sample_rate: u32 = 44100;
    let mut channels: u16 = 2;
    let mut has_video = false;
    let mut has_audio = false;

    for event in iter {
        match event {
            FfmpegEvent::ParsedDuration(d) => {
                duration = Some(Timestamp::from_secs(d.duration));
            }
            FfmpegEvent::ParsedInputStream(stream) => {
                if stream.is_video() {
                    has_video = true;
                    if let Some(video_data) = stream.video_data() {
                        width = video_data.width;
                        height = video_data.height;
                        frame_rate = video_data.fps as f64;
                    }
                    video_codec = stream.format.clone();
                }
                if stream.is_audio() {
                    has_audio = true;
                    if let Some(audio_data) = stream.audio_data() {
                        sample_rate = audio_data.sample_rate;
                        channels = parse_channels(&audio_data.channels);
                    }
                    audio_codec = stream.format.clone();
                }
            }
            FfmpegEvent::Log(_level, msg) => {
                // Parse duration from log messages as fallback
                // Example: "Duration: 00:09:56.46"
                if let Some(dur) = parse_duration_from_log(&msg) {
                    if duration.is_none() {
                        duration = Some(dur);
                    }
                }
            }
            FfmpegEvent::Done | FfmpegEvent::Error(_) => break,
            _ => {}
        }
    }

    child.kill().ok();

    let mut streams = Vec::new();

    if has_video {
        streams.push(StreamInfo::Video(VideoStreamInfo {
            index: 0,
            codec: video_codec,
            width,
            height,
            frame_rate,
            pixel_format: None,
            bit_depth: None,
            color_space: None,
            frame_count: duration.map(|d| (d.as_secs() * frame_rate) as u64),
            duration,
            bitrate: None,
        }));
    }

    if has_audio {
        streams.push(StreamInfo::Audio(AudioStreamInfo {
            index: 1,
            codec: audio_codec,
            sample_rate,
            channels,
            channel_layout: None,
            sample_format: None,
            bit_depth: None,
            duration,
            bitrate: None,
        }));
    }

    let info = MediaInfo {
        format: path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string(),
        duration,
        bitrate: None,
        streams,
        metadata: Vec::new(),
        seekable: true,
        is_live: false,
    };

    Ok((info, width, height))
}

fn parse_channels(channels_str: &str) -> u16 {
    match channels_str.to_lowercase().as_str() {
        "mono" => 1,
        "stereo" => 2,
        "5.1" | "5.1(side)" => 6,
        "7.1" => 8,
        _ => {
            // Try to parse as a number
            channels_str.parse().unwrap_or(2)
        }
    }
}

fn parse_duration_from_log(msg: &str) -> Option<Timestamp> {
    // Look for "Duration: HH:MM:SS.cs"
    if !msg.contains("Duration:") {
        return None;
    }

    let duration_part = msg.split("Duration:").nth(1)?;
    let time_part = duration_part.trim().split(',').next()?.trim();

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;

    let total_secs = hours * 3600.0 + minutes * 60.0 + seconds;
    Some(Timestamp::from_secs(total_secs))
}
