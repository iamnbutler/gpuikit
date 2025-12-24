//! Video Player Example
//!
//! Demonstrates video playback using the gpuikit-media infrastructure.
//!
//! # Requirements
//! - FFmpeg must be installed on your system
//!
//! # Usage
//! ```bash
//! cargo run --bin video_player
//! ```

use ffmpeg_sidecar::download::auto_download;
use gpui::{
    div, px, size, App, AppContext as _, Application, Bounds, Context, Entity, FocusHandle,
    Focusable, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, WindowBounds, WindowOptions,
};
use gpuikit_media::{
    init as init_media, video_view, FfmpegSource, MediaSource, PlaybackState, VideoPlayer,
    VideoPlayerEvent,
};
use gpuikit_theme::{init as init_theme, ActiveTheme, Themeable};

const VIDEO_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../gpuikit/assets/video/big_buck_bunny.mp4"
);

fn main() {
    env_logger::init();

    if let Err(e) = auto_download() {
        eprintln!("Failed to download FFmpeg: {}", e);
        eprintln!("Please install FFmpeg manually and ensure it's in your PATH");
        std::process::exit(1);
    }

    Application::new().run(|cx| {
        init_theme(cx);
        init_media(cx);

        let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| VideoPlayerView::new(window, cx)),
        )
        .expect("Failed to open window");
    });
}

struct VideoPlayerView {
    player: Entity<VideoPlayer>,
    focus_handle: FocusHandle,
    status_message: SharedString,
    _subscription: Subscription,
}

impl VideoPlayerView {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let player = cx.new(|cx| VideoPlayer::new(cx));

        let subscription = cx.subscribe(&player, |this, _player, event, cx| {
            this.handle_player_event(event, cx);
        });

        let mut view = Self {
            player,
            focus_handle: cx.focus_handle(),
            status_message: "Loading video...".into(),
            _subscription: subscription,
        };

        view.load_video(cx);

        view
    }

    fn load_video(&mut self, cx: &mut Context<Self>) {
        match FfmpegSource::open(VIDEO_PATH) {
            Ok(source) => {
                let duration = source
                    .media_info()
                    .duration
                    .map(|d| format!("{:.1}s", d.as_secs()))
                    .unwrap_or_else(|| "unknown".to_string());

                let video_info = source.media_info().video_stream();
                if let Some(video) = video_info {
                    log::info!(
                        "Video loaded: {}x{} @ {:.2} fps",
                        video.width,
                        video.height,
                        video.frame_rate
                    );
                }

                self.player.update(cx, |player, cx| {
                    player.load_source(Box::new(source), cx);
                });

                self.status_message =
                    format!("Loaded: {} (duration: {})", VIDEO_PATH, duration).into();
            }
            Err(e) => {
                self.status_message = format!("Failed to load video: {}", e).into();
                log::error!("Failed to load video: {}", e);
            }
        }
        cx.notify();
    }

    fn handle_player_event(&mut self, event: &VideoPlayerEvent, cx: &mut Context<Self>) {
        match event {
            VideoPlayerEvent::Playing => {
                self.status_message = "Playing".into();
            }
            VideoPlayerEvent::Paused => {
                self.status_message = "Paused".into();
            }
            VideoPlayerEvent::Stopped => {
                self.status_message = "Stopped".into();
            }
            VideoPlayerEvent::PlaybackEnded => {
                self.status_message = "Playback ended".into();
            }
            VideoPlayerEvent::FrameReady => {}
            VideoPlayerEvent::Error(msg) => {
                self.status_message = format!("Error: {}", msg).into();
                log::error!("Player error: {}", msg);
            }
            _ => {}
        }
        cx.notify();
    }

    fn format_time(seconds: f64) -> String {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}:{:02}", mins, secs)
    }
}

impl Focusable for VideoPlayerView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for VideoPlayerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let player = self.player.read(cx);

        let state = player.state();
        let current_frame = player.current_frame().cloned();
        let position = player.position();
        let duration = player.duration();

        let play_button_text = match state {
            PlaybackState::Playing => "⏸ Pause",
            _ => "▶ Play",
        };

        let position_text = format!(
            "{} / {}",
            Self::format_time(position.as_secs()),
            Self::format_time(duration.map(|d| d.as_secs()).unwrap_or(0.0))
        );

        let progress = if let Some(dur) = duration {
            if dur.as_micros() > 0 {
                (position.as_micros() as f32 / dur.as_micros() as f32).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let player_for_play = self.player.clone();
        let player_for_stop = self.player.clone();

        div()
            .id("video-player-root")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg())
            .text_color(theme.fg())
            .child(
                div()
                    .p(px(16.0))
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Video Player"),
            )
            .child(
                div().flex_1().px(px(16.0)).child(
                    div()
                        .size_full()
                        .bg(gpui::rgb(0x000000))
                        .rounded(px(8.0))
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child({
                            let view = video_view(current_frame.clone());
                            if let Some(frame) = &current_frame {
                                view.max_w(px(frame.width as f32))
                                    .max_h(px(frame.height as f32))
                            } else {
                                view.size_full()
                            }
                        }),
                ),
            )
            .child(
                div()
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .h(px(8.0))
                            .w_full()
                            .rounded(px(4.0))
                            .bg(theme.border())
                            .child(
                                div()
                                    .h_full()
                                    .rounded(px(4.0))
                                    .bg(theme.accent())
                                    .w(gpui::relative(progress)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.fg_muted())
                                    .child(self.status_message.clone()),
                            )
                            .child(div().text_sm().child(position_text)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("play-btn")
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .bg(theme.accent())
                                    .text_color(theme.fg())
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .hover(|s| s.opacity(0.9))
                                    .child(play_button_text)
                                    .on_click(move |_, _, cx| {
                                        player_for_play.update(cx, |player, cx| {
                                            player.toggle_playback(cx);
                                        });
                                    }),
                            )
                            .child(
                                div()
                                    .id("stop-btn")
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .bg(theme.surface())
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.border()))
                                    .child("⏹ Stop")
                                    .on_click(move |_, _, cx| {
                                        player_for_stop.update(cx, |player, cx| {
                                            player.stop(cx);
                                        });
                                    }),
                            ),
                    ),
            )
    }
}
