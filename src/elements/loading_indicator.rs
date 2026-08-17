//! Loading indicator component for gpuikit.
//!
//! # Why this does not use `with_animation`
//!
//! gpui's [`AnimationElement`](gpui::AnimationExt) asks the window for another
//! frame for as long as its animation is unfinished, and `.repeat()` is never
//! finished. `Window::request_animation_frame` is `on_next_frame(|_, cx|
//! cx.notify(current_view))`, so a repeating animation re-arms a notify of the
//! *enclosing view* forever: one spinner pins its whole window — sidebar,
//! scroll area and all — at the display refresh rate, whether or not the
//! spinner's glyph actually changed.
//!
//! Indicators therefore share one [`LoadingClock`]. It wakes at the union of
//! the frame boundaries its subscribers asked for — 2–10 times a second rather
//! than 60–120 — and notifies exactly the views that are displaying an
//! indicator, which is the same invalidation gpui was doing, just at the rate
//! the glyphs change. When the last indicator goes away the clock stops
//! entirely and costs nothing until one is rendered again.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, rems, App, AsyncApp, EntityId, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Task, Window,
};
use std::cell::RefCell;
use std::time::Duration;

/// How many of its *own* frame changes a subscriber may miss before the clock
/// forgets it.
///
/// A subscriber is refreshed by rendering, and rendering happens asynchronously
/// with respect to ticks, so expiring on the first missed tick would let one
/// late frame freeze a spinner permanently. Counting the subscriber's own
/// frames rather than global ticks matters too: a view holding only a 500 ms
/// indicator must not be aged out by a 100 ms indicator in some other view
/// driving the clock faster.
const SUBSCRIBER_GRACE_FRAMES: u32 = 4;

/// Create a [`LoadingIndicator`] with the default variant and size.
pub fn loading_indicator() -> LoadingIndicator {
    LoadingIndicator::new()
}

/// The glyph sequence a [`LoadingIndicator`] cycles through.
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingIndicatorVariant {
    /// `.`, `..`, `...`
    #[default]
    Dots,
    /// Dots that fill and then empty again.
    Ellipsis,
    /// A spinning ASCII bar.
    Dash,
    /// Pulsing asterisks.
    Star,
    /// A triangle rotating through the four corners.
    Triangle,
    /// The eight-frame braille spinner.
    Braille,
    /// A 256-frame braille counter.
    BrailleExtended,
}

impl LoadingIndicatorVariant {
    fn frames(&self) -> &'static [&'static str] {
        match self {
            LoadingIndicatorVariant::Dots => &[".  ", ".. ", "..."],
            LoadingIndicatorVariant::Ellipsis => &["   ", ".  ", ".. ", "...", ".. ", ".  "],
            LoadingIndicatorVariant::Dash => &["-", "\\", "|", "/"],
            LoadingIndicatorVariant::Star => &["❊", "❊", "✳︎", "※"],
            LoadingIndicatorVariant::Triangle => &["◢", "◣", "◤", "◥"],
            LoadingIndicatorVariant::Braille => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            LoadingIndicatorVariant::BrailleExtended => &[
                "⡀", "⡁", "⡂", "⡃", "⡄", "⡅", "⡆", "⡇", "⡈", "⡉", "⡊", "⡋", "⡌", "⡍", "⡎", "⡏",
                "⡐", "⡑", "⡒", "⡓", "⡔", "⡕", "⡖", "⡗", "⡘", "⡙", "⡚", "⡛", "⡜", "⡝", "⡞", "⡟",
                "⡠", "⡡", "⡢", "⡣", "⡤", "⡥", "⡦", "⡧", "⡨", "⡩", "⡪", "⡫", "⡬", "⡭", "⡮", "⡯",
                "⡰", "⡱", "⡲", "⡳", "⡴", "⡵", "⡶", "⡷", "⡸", "⡹", "⡺", "⡻", "⡼", "⡽", "⡾", "⡿",
                "⢀", "⢁", "⢂", "⢃", "⢄", "⢅", "⢆", "⢇", "⢈", "⢉", "⢊", "⢋", "⢌", "⢍", "⢎", "⢏",
                "⢐", "⢑", "⢒", "⢓", "⢔", "⢕", "⢖", "⢗", "⢘", "⢙", "⢚", "⢛", "⢜", "⢝", "⢞", "⢟",
                "⢠", "⢡", "⢢", "⢣", "⢤", "⢥", "⢦", "⢧", "⢨", "⢩", "⢪", "⢫", "⢬", "⢭", "⢮", "⢯",
                "⢰", "⢱", "⢲", "⢳", "⢴", "⢵", "⢶", "⢷", "⢸", "⢹", "⢺", "⢻", "⢼", "⢽", "⢾", "⢿",
                "⣀", "⣁", "⣂", "⣃", "⣄", "⣅", "⣆", "⣇", "⣈", "⣉", "⣊", "⣋", "⣌", "⣍", "⣎", "⣏",
                "⣐", "⣑", "⣒", "⣓", "⣔", "⣕", "⣖", "⣗", "⣘", "⣙", "⣚", "⣛", "⣜", "⣝", "⣞", "⣟",
                "⣠", "⣡", "⣢", "⣣", "⣤", "⣥", "⣦", "⣧", "⣨", "⣩", "⣪", "⣫", "⣬", "⣭", "⣮", "⣯",
                "⣰", "⣱", "⣲", "⣳", "⣴", "⣵", "⣶", "⣷", "⣸", "⣹", "⣺", "⣻", "⣼", "⣽", "⣾", "⣿",
            ],
        }
    }

    fn duration(&self) -> Duration {
        match self {
            LoadingIndicatorVariant::Dots => Duration::from_millis(1500),
            LoadingIndicatorVariant::Ellipsis => Duration::from_millis(1800),
            LoadingIndicatorVariant::Dash => Duration::from_millis(400),
            LoadingIndicatorVariant::Star => Duration::from_millis(1000),
            LoadingIndicatorVariant::Triangle => Duration::from_millis(1200),
            LoadingIndicatorVariant::Braille => Duration::from_millis(1000),
            LoadingIndicatorVariant::BrailleExtended => Duration::from_millis(30000),
        }
    }

    /// How long one glyph is on screen — the interval at which a view showing
    /// this variant needs to be redrawn, and nothing faster.
    fn frame_period(&self) -> Duration {
        self.duration() / self.frames().len() as u32
    }

    fn char_width(&self) -> usize {
        self.frames()
            .iter()
            .map(|f| f.chars().count())
            .max()
            .unwrap_or(1)
    }
}

/// The text size a [`LoadingIndicator`] renders at.
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingIndicatorSize {
    /// `text_xs`
    XSmall,
    /// `text_sm`
    Small,
    /// `text_base`
    #[default]
    Medium,
    /// `text_xl`
    Large,
}

/// A text spinner.
///
/// Frames come from the shared [`LoadingClock`] rather than from a per-element
/// animation, so an indicator costs its window one redraw per glyph rather than
/// one per display refresh. See the [module docs](self).
#[derive(IntoElement)]
pub struct LoadingIndicator {
    variant: LoadingIndicatorVariant,
    size: LoadingIndicatorSize,
    color: Option<Hsla>,
    playing: bool,
}

impl LoadingIndicator {
    /// Create an indicator with the default variant and size.
    pub fn new() -> Self {
        Self {
            variant: LoadingIndicatorVariant::default(),
            size: LoadingIndicatorSize::default(),
            color: None,
            playing: true,
        }
    }

    /// Set the glyph sequence.
    pub fn variant(mut self, variant: LoadingIndicatorVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Use [`LoadingIndicatorVariant::Dots`].
    pub fn dots(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Dots;
        self
    }

    /// Use [`LoadingIndicatorVariant::Ellipsis`].
    pub fn ellipsis(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Ellipsis;
        self
    }

    /// Use [`LoadingIndicatorVariant::Dash`].
    pub fn dash(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Dash;
        self
    }

    /// Use [`LoadingIndicatorVariant::Star`].
    pub fn star(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Star;
        self
    }

    /// Use [`LoadingIndicatorVariant::Triangle`].
    pub fn triangle(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Triangle;
        self
    }

    /// Use [`LoadingIndicatorVariant::Braille`].
    pub fn braille(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::Braille;
        self
    }

    /// Use [`LoadingIndicatorVariant::BrailleExtended`].
    pub fn braille_extended(mut self) -> Self {
        self.variant = LoadingIndicatorVariant::BrailleExtended;
        self
    }

    /// Set the text size.
    pub fn size(mut self, size: LoadingIndicatorSize) -> Self {
        self.size = size;
        self
    }

    /// Use [`LoadingIndicatorSize::XSmall`].
    pub fn xsmall(mut self) -> Self {
        self.size = LoadingIndicatorSize::XSmall;
        self
    }

    /// Use [`LoadingIndicatorSize::Small`].
    pub fn small(mut self) -> Self {
        self.size = LoadingIndicatorSize::Small;
        self
    }

    /// Use [`LoadingIndicatorSize::Medium`].
    pub fn medium(mut self) -> Self {
        self.size = LoadingIndicatorSize::Medium;
        self
    }

    /// Use [`LoadingIndicatorSize::Large`].
    pub fn large(mut self) -> Self {
        self.size = LoadingIndicatorSize::Large;
        self
    }

    /// Override the glyph colour. Defaults to the theme's accent.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Whether the indicator advances. Defaults to `true`.
    ///
    /// A paused indicator renders its first frame and subscribes to nothing, so
    /// it costs its window no redraws at all. [`App::reduce_motion`] has the
    /// same effect regardless of this setting.
    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or_else(|| theme.accent());

        let frames = self.variant.frames();
        let glyph = if self.playing && !cx.reduce_motion() {
            let period = self.variant.frame_period();
            // Legal here: `draw_roots` enters `DrawPhase::Prepaint` before the
            // root view renders, and every view's element wraps its render in
            // `with_rendered_view`, so the enclosing view is on the stack for
            // the whole of `RenderOnce::render`.
            let view = window.current_view();
            let elapsed = LoadingClock::subscribe(view, period, cx);
            frames[(frame_index(elapsed, period) % frames.len() as u64) as usize]
        } else {
            frames[0]
        };

        let size = self.size;
        let width_rems = self.variant.char_width() as f32 * 0.6;

        div()
            .text_color(color)
            .flex_none()
            .min_w(rems(width_rems))
            .text_center()
            .when(matches!(size, LoadingIndicatorSize::XSmall), |this| {
                this.text_xs()
            })
            .when(matches!(size, LoadingIndicatorSize::Small), |this| {
                this.text_sm()
            })
            .when(matches!(size, LoadingIndicatorSize::Medium), |this| {
                this.text_base()
            })
            .when(matches!(size, LoadingIndicatorSize::Large), |this| {
                this.text_xl()
            })
            // `SharedString::new_static` on a `&'static str` that fits inline:
            // no allocation, per indicator per frame.
            .child(SharedString::new_static(glyph))
    }
}

/// Which frame of a `period`-long cycle `elapsed` falls in.
fn frame_index(elapsed: Duration, period: Duration) -> u64 {
    debug_assert!(!period.is_zero());
    (elapsed.as_nanos() / period.as_nanos().max(1)) as u64
}

/// One view, wanting one frame period.
///
/// A view showing several indicators registers once per distinct period; the
/// notifications a tick produces are deduplicated, so it is still one redraw.
struct Subscriber {
    view: EntityId,
    period: Duration,
    last_frame: u64,
    /// Frame changes since this subscriber last re-registered by rendering.
    idle_frames: u32,
}

/// The timeline every [`LoadingIndicator`] reads its frame from.
///
/// Advanced by an accumulated `Duration` rather than a wall-clock delta: the
/// driver only ever sleeps to a frame boundary, so adding exactly the interval
/// it scheduled keeps the timeline on those boundaries and makes [`tick`] a
/// pure function that a test can step with no clock at all.
///
/// [`tick`]: LoadingClock::tick
#[derive(Default)]
struct LoadingClock {
    elapsed: Duration,
    subscribers: Vec<Subscriber>,
    /// Whether a driver task is live. Kept separately from `driver` because the
    /// driver cannot clear `driver` itself — that would drop a running task
    /// from inside its own future.
    running: bool,
    driver: Option<Task<()>>,
}

thread_local! {
    static LOADING_CLOCK: RefCell<LoadingClock> = RefCell::new(LoadingClock::default());
}

/// Run `f` against the shared clock, releasing the borrow before returning.
///
/// Every caller goes through this: the borrow must not still be live when
/// `cx.notify` re-enters gpui, and that is a runtime panic rather than a
/// compile error.
fn with_clock<R>(f: impl FnOnce(&mut LoadingClock) -> R) -> R {
    LOADING_CLOCK.with(|clock| f(&mut clock.borrow_mut()))
}

impl LoadingClock {
    /// Register `view` as showing an indicator with the given frame period, and
    /// return the shared timeline's current position.
    ///
    /// Starts the driver if it is not already running.
    fn subscribe(view: EntityId, period: Duration, cx: &mut App) -> Duration {
        let (elapsed, start_driver) = with_clock(|clock| {
            clock.register(view, period);
            let start_driver = !clock.running;
            clock.running = true;
            (clock.elapsed, start_driver)
        });

        if start_driver {
            let driver = cx.spawn(async move |cx| Self::drive(cx).await);
            with_clock(|clock| clock.driver = Some(driver));
        }

        elapsed
    }

    fn register(&mut self, view: EntityId, period: Duration) {
        if let Some(existing) = self
            .subscribers
            .iter_mut()
            .find(|s| s.view == view && s.period == period)
        {
            existing.idle_frames = 0;
            return;
        }

        self.subscribers.push(Subscriber {
            view,
            period,
            last_frame: frame_index(self.elapsed, period),
            idle_frames: 0,
        });
    }

    /// How long until the first frame boundary any subscriber is waiting on.
    ///
    /// The union of the registered periods, not a shared quantized rate: a
    /// fixed 10 Hz tick would round Braille's 125 ms frames into a visible
    /// 100/200 ms limp, and scheduling against the union costs nothing extra.
    ///
    /// `None` when nothing is subscribed, which is how the driver stops.
    fn next_interval(&self) -> Option<Duration> {
        let elapsed = self.elapsed.as_nanos();
        self.subscribers
            .iter()
            .map(|s| {
                let period = s.period.as_nanos().max(1);
                period - (elapsed % period)
            })
            .min()
            .map(|nanos| Duration::from_nanos(nanos as u64))
    }

    /// Advance the timeline by `interval` and return the views whose glyphs
    /// changed, dropping subscribers that have stopped re-registering.
    fn tick(&mut self, interval: Duration) -> Vec<EntityId> {
        self.elapsed += interval;
        let elapsed = self.elapsed;

        let mut due: Vec<EntityId> = Vec::new();
        self.subscribers.retain_mut(|subscriber| {
            let frame = frame_index(elapsed, subscriber.period);
            if frame == subscriber.last_frame {
                return true;
            }
            subscriber.last_frame = frame;

            subscriber.idle_frames += 1;
            if subscriber.idle_frames > SUBSCRIBER_GRACE_FRAMES {
                return false;
            }

            if !due.contains(&subscriber.view) {
                due.push(subscriber.view);
            }
            true
        });

        due
    }

    /// Sleep to each frame boundary in turn, notifying the views that reach one.
    ///
    /// Returns — rather than clearing `driver` — when nothing is subscribed:
    /// clearing it would drop this task from inside its own future. The
    /// finished `Task` is simply replaced by the next `subscribe`.
    async fn drive(cx: &mut AsyncApp) {
        while let Some(interval) = with_clock(|clock| clock.next_interval()) {
            cx.background_executor().timer(interval).await;

            // The borrow ends here, before `cx.notify` re-enters gpui and
            // renders — which subscribes, and would find the clock borrowed.
            let due = with_clock(|clock| clock.tick(interval));
            if due.is_empty() {
                continue;
            }

            cx.update(|cx| {
                for view in due {
                    cx.notify(view);
                }
            });
        }

        with_clock(|clock| clock.running = false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, Context, Render, TestAppContext};

    /// The clock outlives an individual `App`, so a test states its own
    /// starting point rather than inheriting one.
    fn reset_clock() {
        with_clock(|clock| {
            clock.elapsed = Duration::ZERO;
            clock.subscribers.clear();
            clock.driver = None;
            clock.running = false;
        });
    }

    fn view(id: u64) -> EntityId {
        EntityId::from(id)
    }

    struct Indicators;

    impl Render for Indicators {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size(px(200.))
                .child(loading_indicator().dash())
                .child(loading_indicator().braille())
                .child(loading_indicator().dots())
        }
    }

    struct Paused;

    impl Render for Paused {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size(px(200.))
                .child(loading_indicator().braille().playing(false))
        }
    }

    /// The whole point of the change: an indicator must leave no next-frame
    /// callback behind. Reattaching a repeating animation to any of these fails
    /// here.
    #[gpui::test]
    fn indicators_do_not_request_a_frame_per_display_frame(cx: &mut TestAppContext) {
        reset_clock();
        cx.update(crate::theme::init);

        let (_view, cx) = cx.add_window_view(|_window, _cx| Indicators);

        let requested = cx.update(|window, cx| window.simulate_next_frame(cx));
        assert_eq!(requested, 0, "an indicator asked for another frame");
    }

    /// The counterpart: "quiet" must not be reachable by simply not animating.
    #[gpui::test]
    fn the_shared_clock_advances_over_time(cx: &mut TestAppContext) {
        reset_clock();
        cx.update(crate::theme::init);

        let cx = cx.add_window_view(|_window, _cx| Indicators).1;
        assert!(with_clock(|clock| clock.running), "the clock never started");
        assert_eq!(with_clock(|clock| clock.elapsed), Duration::ZERO);

        // Dash's frame period is 100 ms, the shortest of the three.
        cx.executor().advance_clock(Duration::from_millis(300));

        assert!(
            with_clock(|clock| clock.elapsed) >= Duration::from_millis(300),
            "the shared timeline did not move"
        );
    }

    #[gpui::test]
    fn a_paused_indicator_subscribes_to_nothing(cx: &mut TestAppContext) {
        reset_clock();
        cx.update(crate::theme::init);

        cx.add_window_view(|_window, _cx| Paused);

        assert!(with_clock(|clock| clock.subscribers.is_empty()));
        assert!(!with_clock(|clock| clock.running));
    }

    #[test]
    fn every_variant_divides_its_cycle_evenly() {
        // The timeline only lands exactly on frame boundaries if a period is an
        // exact division of its cycle.
        for variant in [
            LoadingIndicatorVariant::Dots,
            LoadingIndicatorVariant::Ellipsis,
            LoadingIndicatorVariant::Dash,
            LoadingIndicatorVariant::Star,
            LoadingIndicatorVariant::Triangle,
            LoadingIndicatorVariant::Braille,
            LoadingIndicatorVariant::BrailleExtended,
        ] {
            let frames = variant.frames().len() as u32;
            assert_eq!(
                variant.frame_period() * frames,
                variant.duration(),
                "{variant:?} loses time to rounding"
            );
        }
    }

    #[test]
    fn a_tick_wakes_only_the_views_whose_glyph_changed() {
        let (fast, slow) = (view(1), view(2));
        let mut clock = LoadingClock::default();
        clock.register(fast, Duration::from_millis(100));
        clock.register(slow, Duration::from_millis(250));

        // The union of the two periods, not a rate they both have to share.
        assert_eq!(clock.next_interval(), Some(Duration::from_millis(100)));
        assert_eq!(clock.tick(Duration::from_millis(100)), vec![fast]);
        clock.register(fast, Duration::from_millis(100));

        assert_eq!(clock.next_interval(), Some(Duration::from_millis(100)));
        assert_eq!(clock.tick(Duration::from_millis(100)), vec![fast]);
        clock.register(fast, Duration::from_millis(100));

        // 250 ms comes before 300 ms, and only the slow view has changed.
        assert_eq!(clock.next_interval(), Some(Duration::from_millis(50)));
        assert_eq!(clock.tick(Duration::from_millis(50)), vec![slow]);
    }

    #[test]
    fn a_view_showing_several_indicators_is_woken_once() {
        let both = view(1);
        let mut clock = LoadingClock::default();
        clock.register(both, Duration::from_millis(100));
        clock.register(both, Duration::from_millis(50));

        assert_eq!(clock.tick(Duration::from_millis(100)), vec![both]);
    }

    #[test]
    fn a_subscriber_that_stops_rendering_expires_and_the_clock_goes_quiet() {
        let view = view(1);
        let period = Duration::from_millis(100);
        let mut clock = LoadingClock::default();
        clock.register(view, period);

        // Re-registering — which is what rendering does — keeps it alive
        // indefinitely.
        for _ in 0..(SUBSCRIBER_GRACE_FRAMES * 3) {
            assert_eq!(clock.tick(period), vec![view]);
            clock.register(view, period);
        }

        // Left alone it survives the grace period, and then goes.
        for _ in 0..SUBSCRIBER_GRACE_FRAMES {
            assert_eq!(clock.tick(period), vec![view]);
        }
        assert!(clock.tick(period).is_empty());
        assert_eq!(clock.next_interval(), None, "the clock kept ticking");
    }

    #[test]
    fn a_slow_subscriber_is_not_aged_out_by_a_fast_one() {
        let mut clock = LoadingClock::default();
        clock.register(view(1), Duration::from_millis(500));

        // Ten ticks of somebody else's 100 ms clock: two of this view's own
        // frames, well inside its grace.
        for _ in 0..10 {
            clock.tick(Duration::from_millis(100));
        }

        assert_eq!(clock.subscribers.len(), 1);
    }
}
