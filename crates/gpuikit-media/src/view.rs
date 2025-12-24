//! Video player view component for rendering video with controls.
//!
//! This module provides [`VideoPlayerView`], a complete video player UI component
//! that wraps a [`VideoPlayer`] entity and provides standard playback controls.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit_media::view::{VideoPlayerView, video_player_view};
//!
//! // In your render method:
//! fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!     video_player_view(self.player.clone())
//!         .show_controls(true)
//!         .size_full()
//! }
//! ```

use gpui::{
    div, prelude::FluentBuilder, px, App, Bounds, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, ObjectFit, ParentElement,
    Pixels, Refineable, RenderOnce, StatefulInteractiveElement, Style, StyleRefinement, Styled,
    Window,
};
use gpuikit_theme::{ActiveTheme, Themeable};

use crate::frame::VideoFrame;
use crate::player::{PlaybackState, VideoPlayer};
use crate::{MediaDuration, Timestamp, Volume};

/// Creates a new video player view.
pub fn video_player_view(player: Entity<VideoPlayer>) -> VideoPlayerView {
    VideoPlayerView::new(player)
}

/// A complete video player UI component.
///
/// Renders the current video frame and optional playback controls including:
/// - Play/pause button
/// - Progress/seek bar
/// - Current time / duration display
/// - Volume control
/// - Fullscreen toggle (optional)
#[derive(IntoElement)]
pub struct VideoPlayerView {
    player: Entity<VideoPlayer>,
    show_controls: bool,
    controls_visible: bool,
    auto_hide_controls: bool,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

impl VideoPlayerView {
    /// Create a new video player view wrapping the given player entity.
    pub fn new(player: Entity<VideoPlayer>) -> Self {
        Self {
            player,
            show_controls: true,
            controls_visible: true,
            auto_hide_controls: true,
            object_fit: ObjectFit::Contain,
            style: StyleRefinement::default(),
        }
    }

    /// Set whether to show playback controls.
    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Set whether controls should auto-hide during playback.
    pub fn auto_hide_controls(mut self, auto_hide: bool) -> Self {
        self.auto_hide_controls = auto_hide;
        self
    }

    /// Set the object fit mode for the video.
    pub fn object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self
    }
}

impl RenderOnce for VideoPlayerView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let player = self.player.read(cx);
        let state = player.state();
        let current_frame = player.current_frame().cloned();
        let duration_info = player.duration_info();
        let volume = player.volume();
        let is_muted = player.is_muted();
        let player_entity = self.player.clone();

        div()
            .id("video-player-view")
            .relative()
            .overflow_hidden()
            .bg(cx.theme().surface())
            .child(VideoSurface {
                frame: current_frame,
                object_fit: self.object_fit,
                style: StyleRefinement::default(),
            })
            .when(self.show_controls, |this| {
                this.child(VideoControls {
                    player: player_entity,
                    state,
                    duration_info,
                    volume,
                    is_muted,
                    visible: self.controls_visible,
                })
            })
    }
}

impl Styled for VideoPlayerView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Internal element for rendering the video frame.
struct VideoSurface {
    frame: Option<VideoFrame>,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

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
        style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Fraction(1.0));
        style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Fraction(1.0));
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
        let Some(frame) = &self.frame else {
            return;
        };

        let frame_size = frame.size_device_pixels();
        let fitted_bounds = self.object_fit.get_bounds(bounds, frame_size);

        #[cfg(target_os = "macos")]
        if let Some(buffer) = frame.as_cv_pixel_buffer() {
            window.paint_surface(fitted_bounds, buffer.clone());
            return;
        }

        if let Some(render_image) = frame.to_render_image() {
            window
                .paint_image(
                    fitted_bounds,
                    gpui::Corners::default(),
                    render_image,
                    0,
                    false,
                )
                .ok();
        }
    }
}

impl IntoElement for VideoSurface {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Playback controls overlay.
#[derive(IntoElement)]
struct VideoControls {
    player: Entity<VideoPlayer>,
    state: PlaybackState,
    duration_info: MediaDuration,
    volume: Volume,
    is_muted: bool,
    visible: bool,
}

impl RenderOnce for VideoControls {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let player = self.player.clone();
        let state = self.state;
        let duration_info = self.duration_info;
        let progress = duration_info.progress();

        div()
            .id("video-controls")
            .absolute()
            .bottom_0()
            .left_0()
            .right_0()
            .p(px(8.0))
            .bg(theme.surface().opacity(0.8))
            .when(!self.visible, |this| this.opacity(0.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .child(PlayPauseButton {
                        player: player.clone(),
                        state,
                    })
                    .child(div().flex_1().child(ProgressBar {
                        player: player.clone(),
                        progress,
                    }))
                    .child(TimeDisplay {
                        current: duration_info.current,
                        total: duration_info.total,
                    })
                    .child(VolumeButton {
                        player: player.clone(),
                        volume: self.volume,
                        is_muted: self.is_muted,
                    }),
            )
    }
}

/// Play/pause toggle button.
#[derive(IntoElement)]
struct PlayPauseButton {
    player: Entity<VideoPlayer>,
    state: PlaybackState,
}

impl RenderOnce for PlayPauseButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let player = self.player.clone();
        let is_playing = self.state == PlaybackState::Playing;

        let icon = if is_playing { "⏸" } else { "▶" };

        div()
            .id("play-pause-button")
            .flex()
            .items_center()
            .justify_center()
            .w(px(32.0))
            .h(px(32.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|style| style.bg(theme.surface().opacity(0.8)))
            .active(|style| style.bg(theme.surface().opacity(0.6)))
            .text_color(theme.fg())
            .child(icon)
            .on_click(move |_event, _window, cx| {
                player.update(cx, |player, cx| {
                    player.toggle_playback(cx);
                });
            })
    }
}

/// Progress/seek bar.
#[derive(IntoElement)]
struct ProgressBar {
    player: Entity<VideoPlayer>,
    progress: f64,
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let player = self.player.clone();
        let progress = self.progress;

        div()
            .id("progress-bar")
            .h(px(8.0))
            .w_full()
            .rounded(px(4.0))
            .bg(theme.border())
            .cursor_pointer()
            .child(
                div()
                    .h_full()
                    .rounded(px(4.0))
                    .bg(theme.accent())
                    .w(gpui::relative(progress as f32)),
            )
            .on_click(move |_event, _window, cx| {
                // TODO: Calculate click position relative to progress bar bounds
                // For now, just toggle play/pause as a simpler interaction
                player.update(cx, |player, cx| {
                    player.toggle_playback(cx);
                });
            })
    }
}

/// Time display (current / total).
#[derive(IntoElement)]
struct TimeDisplay {
    current: Timestamp,
    total: Timestamp,
}

impl RenderOnce for TimeDisplay {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let current_str = format_duration(self.current);
        let total_str = format_duration(self.total);

        div()
            .flex()
            .items_center()
            .text_sm()
            .text_color(theme.fg_muted())
            .child(format!("{} / {}", current_str, total_str))
    }
}

/// Volume control button.
#[derive(IntoElement)]
struct VolumeButton {
    player: Entity<VideoPlayer>,
    volume: Volume,
    is_muted: bool,
}

impl RenderOnce for VolumeButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let player = self.player.clone();

        let icon = if self.is_muted || self.volume.0 == 0.0 {
            "🔇"
        } else if self.volume.0 < 0.5 {
            "🔈"
        } else {
            "🔊"
        };

        div()
            .id("volume-button")
            .flex()
            .items_center()
            .justify_center()
            .w(px(32.0))
            .h(px(32.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|style| style.bg(theme.surface().opacity(0.8)))
            .active(|style| style.bg(theme.surface().opacity(0.6)))
            .text_color(theme.fg())
            .child(icon)
            .on_click(move |_event, _window, cx| {
                player.update(cx, |player, cx| {
                    player.toggle_mute(cx);
                });
            })
    }
}

/// Format a timestamp as MM:SS or HH:MM:SS.
fn format_duration(timestamp: Timestamp) -> String {
    let total_seconds = timestamp.as_secs().max(0.0) as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// A minimal video view without controls.
///
/// Use this when you want to render just the video frame without any UI.
#[derive(IntoElement)]
pub struct VideoView {
    frame: Option<VideoFrame>,
    object_fit: ObjectFit,
    style: StyleRefinement,
}

impl VideoView {
    /// Create a new video view with the given frame.
    pub fn new(frame: Option<VideoFrame>) -> Self {
        Self {
            frame,
            object_fit: ObjectFit::Contain,
            style: StyleRefinement::default(),
        }
    }

    /// Set the object fit mode.
    pub fn object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self
    }
}

impl RenderOnce for VideoView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        VideoSurface {
            frame: self.frame,
            object_fit: self.object_fit,
            style: self.style,
        }
    }
}

impl Styled for VideoView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Creates a minimal video view without controls.
pub fn video_view(frame: Option<VideoFrame>) -> VideoView {
    VideoView::new(frame)
}
