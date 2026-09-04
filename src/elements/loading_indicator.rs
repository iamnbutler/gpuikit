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
//! spinner's frame actually changed.
//!
//! Indicators therefore share one `LoadingClock`. It wakes at the union of
//! the frame boundaries its subscribers asked for — 2–10 times a second rather
//! than 60–120 — and notifies exactly the views that are displaying an
//! indicator, which is the same invalidation gpui was doing, just at the rate
//! the frames change. When the last indicator goes away the clock stops
//! entirely and costs nothing until one is rendered again.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    App, AsyncApp, EntityId, Hsla, IntoElement, ParentElement, Rems, RenderOnce, SharedString,
    Styled, Task, Window, div, prelude::FluentBuilder, rems,
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

/// One frame of an indicator: either characters, or lit dots on a grid.
///
/// Four of the seven variants used to be characters too — `❊✳※`, `◢◣◤◥` and
/// the braille blocks. That made them a bet on the consumer's font, and the
/// bet loses on gpui's own web platform, which bundles IBM Plex Sans and Lilex
/// and calls `new_without_system_fonts`: neither font has U+2733, U+25E2 or
/// anything in the braille block, there is no fallback to fall back to, and
/// all four rendered as empty boxes. Drawing them removes the bet. A spinner
/// in a GPU toolkit has no business depending on text at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frame {
    /// Characters. Kept for the variants whose whole identity *is* the
    /// characters, and which only ever use ASCII: a dot is a dot everywhere.
    Text(&'static str),
    /// Lit dots on a `cols` x `rows` grid, one bit per cell, row-major from
    /// the top left.
    Dots { cols: u16, rows: u16, mask: u16 },
}

/// A braille byte as a row-major mask on the 2x4 grid its dots are drawn on.
///
/// The braille frames below stay written as the bytes they came from — `0xFE`
/// is `⣾` — because that is what keeps the eight-frame spinner recognisable as
/// the one every terminal draws. Braille numbers its dots down the left
/// column, then down the right, then the low pair, so the byte is not itself a
/// row-major bitmap and this is the translation.
const fn braille(byte: u8) -> u16 {
    /// `(bit in the braille byte, cell index on the 2-wide, 4-tall grid)`.
    const CELLS: [(u8, u16); 8] = [
        (0, 0), // dot 1: left column, row 0
        (1, 2), // dot 2: left, row 1
        (2, 4), // dot 3: left, row 2
        (3, 1), // dot 4: right, row 0
        (4, 3), // dot 5: right, row 1
        (5, 5), // dot 6: right, row 2
        (6, 6), // dot 7: left, row 3
        (7, 7), // dot 8: right, row 3
    ];

    let mut mask = 0u16;
    let mut i = 0;
    while i < CELLS.len() {
        let (bit, cell) = CELLS[i];
        if byte & (1 << bit) != 0 {
            mask |= 1 << cell;
        }
        i += 1;
    }
    mask
}

/// A 2x4 dot frame from a braille byte.
const fn braille_frame(byte: u8) -> Frame {
    Frame::Dots {
        cols: 2,
        rows: 4,
        mask: braille(byte),
    }
}

/// The eight-frame braille spinner: one dot dark, walking the ring.
const BRAILLE_SPINNER: [Frame; 8] = [
    braille_frame(0xFE),
    braille_frame(0xFD),
    braille_frame(0xFB),
    braille_frame(0xBF),
    braille_frame(0x7F),
    braille_frame(0xDF),
    braille_frame(0xEF),
    braille_frame(0xF7),
];

/// Every non-empty braille pattern, in order — a binary counter on eight dots.
///
/// 255 frames, not 256: mask 0 draws nothing, and a spinner that vanishes for
/// one frame per cycle reads as a dropped frame rather than as a count. The
/// glyph version of this claimed 256 in its own doc comment and listed 192.
const BRAILLE_COUNTER: [Frame; 255] = {
    let mut frames = [braille_frame(1); 255];
    let mut i = 0;
    while i < frames.len() {
        frames[i] = braille_frame((i + 1) as u8);
        i += 1;
    }
    frames
};

/// The frame sequence a [`LoadingIndicator`] cycles through.
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadingIndicatorVariant {
    /// `.`, `..`, `...`
    #[default]
    Dots,
    /// Dots that fill and then empty again.
    Ellipsis,
    /// A spinning ASCII bar.
    Dash,
    /// A drawn star, twinkling: the centre, a plus, a cross, a plus.
    Star,
    /// A drawn triangle rotating through the four corners — three lit dots of
    /// a 2x2, with the dark one walking clockwise.
    Triangle,
    /// The eight-frame braille spinner, drawn as dots rather than typed.
    Braille,
    /// A 255-frame braille counter, drawn as dots rather than typed.
    BrailleExtended,
}

impl LoadingIndicatorVariant {
    fn frames(&self) -> &'static [Frame] {
        match self {
            LoadingIndicatorVariant::Dots => {
                &[Frame::Text(".  "), Frame::Text(".. "), Frame::Text("...")]
            }
            LoadingIndicatorVariant::Ellipsis => &[
                Frame::Text("   "),
                Frame::Text(".  "),
                Frame::Text(".. "),
                Frame::Text("..."),
                Frame::Text(".. "),
                Frame::Text(".  "),
            ],
            LoadingIndicatorVariant::Dash => &[
                Frame::Text("-"),
                Frame::Text("\\"),
                Frame::Text("|"),
                Frame::Text("/"),
            ],
            // A 3x3: the centre alone, a plus, a cross, a plus. Read as bits
            // from the top left, so `0b010_111_010` is the middle row full and
            // the middle column full — a plus.
            LoadingIndicatorVariant::Star => &[
                Frame::Dots {
                    cols: 3,
                    rows: 3,
                    mask: 0b000_010_000,
                },
                Frame::Dots {
                    cols: 3,
                    rows: 3,
                    mask: 0b010_111_010,
                },
                Frame::Dots {
                    cols: 3,
                    rows: 3,
                    mask: 0b101_010_101,
                },
                Frame::Dots {
                    cols: 3,
                    rows: 3,
                    mask: 0b010_111_010,
                },
            ],
            // Three lit dots of a 2x2 make a right triangle; which corner is
            // dark is which way it points. The dark corner walks clockwise
            // from the top left, so the triangle rotates.
            LoadingIndicatorVariant::Triangle => &[
                Frame::Dots {
                    cols: 2,
                    rows: 2,
                    mask: 0b11_10,
                },
                Frame::Dots {
                    cols: 2,
                    rows: 2,
                    mask: 0b11_01,
                },
                Frame::Dots {
                    cols: 2,
                    rows: 2,
                    mask: 0b01_11,
                },
                Frame::Dots {
                    cols: 2,
                    rows: 2,
                    mask: 0b10_11,
                },
            ],
            LoadingIndicatorVariant::Braille => &BRAILLE_SPINNER,
            LoadingIndicatorVariant::BrailleExtended => &BRAILLE_COUNTER,
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
            // 100ms a frame. Not the round 30s the glyph version named:
            // `every_variant_divides_its_cycle_evenly` requires the cycle to
            // be an exact multiple of the frame count, and 255 does not divide
            // 30s in whole nanoseconds. 25.5s does, and reads as a counter.
            LoadingIndicatorVariant::BrailleExtended => Duration::from_millis(25_500),
        }
    }

    /// How long one frame is on screen — the interval at which a view showing
    /// this variant needs to be redrawn, and nothing faster.
    fn frame_period(&self) -> Duration {
        self.duration() / self.frames().len() as u32
    }

    /// The width the indicator reserves, in multiples of its own height, so a
    /// spinner does not shift the text beside it as its frames change width.
    fn width_ratio(&self) -> f32 {
        match self.frames().first() {
            // 0.6em per character is the ratio the glyph version used.
            Some(Frame::Text(_)) => {
                let chars = self
                    .frames()
                    .iter()
                    .map(|frame| match frame {
                        Frame::Text(text) => text.chars().count(),
                        Frame::Dots { .. } => 1,
                    })
                    .max()
                    .unwrap_or(1);
                chars as f32 * 0.6
            }
            // A grid is as wide as its columns are tall.
            Some(Frame::Dots { cols, rows, .. }) => *cols as f32 / *rows as f32,
            None => 1.0,
        }
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

/// A spinner.
///
/// Frames come from the shared `LoadingClock` rather than from a per-element
/// animation, so an indicator costs its window one redraw per frame rather
/// than one per display refresh. See the [module docs](self).
///
/// Three variants are text and four are drawn. The drawn ones are drawn
/// because they used to be text: their glyphs are absent from the fonts gpui's
/// web platform bundles, and an indicator that renders as an empty box on a
/// supported target is not an indicator.
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

    /// Set the frame sequence.
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

    /// Override the colour of the text or the dots. Defaults to the theme's
    /// accent.
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

impl LoadingIndicatorSize {
    /// The indicator's height, and the font size a `Text` frame draws at.
    ///
    /// Named in rems rather than left to `text_xs()` and friends because a
    /// `Dots` frame has to be *measured*, not just styled: the grid is sized
    /// from this so a drawn frame occupies the same box a typed one would and
    /// the two families line up beside the same paragraph.
    fn height(self) -> Rems {
        match self {
            // The values `text_xs`, `text_sm`, `text_base` and `text_xl` set.
            LoadingIndicatorSize::XSmall => rems(0.75),
            LoadingIndicatorSize::Small => rems(0.875),
            LoadingIndicatorSize::Medium => rems(1.0),
            LoadingIndicatorSize::Large => rems(1.25),
        }
    }
}

/// One frame of lit dots, drawn.
///
/// The grid fills a `height`-tall box; a cell is that height over the row
/// count, and the dot is most of a cell, which leaves the gap between dots
/// without anyone naming a gap. Unlit cells are drawn as empty boxes rather
/// than skipped, so every frame of a variant is the same size and the grid
/// does not jitter as dots come and go.
fn dot_grid(cols: u16, rows: u16, mask: u16, height: Rems, color: Hsla) -> impl IntoElement {
    let cell = height / rows as f32;
    let dot = cell * 0.72;

    div()
        .flex()
        .flex_col()
        .flex_none()
        .h(height)
        .children((0..rows).map(|row| {
            div()
                .flex()
                .flex_row()
                .h(cell)
                .children((0..cols).map(move |col| {
                    let lit = mask & (1 << (row * cols + col)) != 0;
                    div()
                        .w(cell)
                        .h(cell)
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(lit, |this| {
                            this.child(div().w(dot).h(dot).rounded_full().bg(color))
                        })
                }))
        }))
}

impl RenderOnce for LoadingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or_else(|| theme.accent());

        let frames = self.variant.frames();
        let frame = if self.playing && !cx.reduce_motion() {
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

        let height = self.size.height();

        div()
            .text_color(color)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .min_w(height * self.variant.width_ratio())
            .text_size(height)
            .line_height(height)
            .map(|this| match frame {
                // `SharedString::new_static` on a `&'static str` that fits
                // inline: no allocation, per indicator per frame.
                Frame::Text(text) => this.child(SharedString::new_static(text)),
                Frame::Dots { cols, rows, mask } => {
                    this.child(dot_grid(cols, rows, mask, height, color))
                }
            })
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

    /// Advance the timeline by `interval` and return the views whose frames
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
    use gpui::{Context, Render, TestAppContext, px};

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

    fn all_variants() -> [LoadingIndicatorVariant; 7] {
        [
            LoadingIndicatorVariant::Dots,
            LoadingIndicatorVariant::Ellipsis,
            LoadingIndicatorVariant::Dash,
            LoadingIndicatorVariant::Star,
            LoadingIndicatorVariant::Triangle,
            LoadingIndicatorVariant::Braille,
            LoadingIndicatorVariant::BrailleExtended,
        ]
    }

    /// The defect that made four variants drawn ones: they were built from
    /// `❊✳※`, `◢◣◤◥` and the braille block, none of which is in either font
    /// gpui's web platform bundles, and that platform loads no system fonts.
    /// All four rendered as empty boxes in the browser.
    ///
    /// ASCII is the line because it is the only repertoire the crate can
    /// promise across a consumer's font choices. A variant that wants a shape
    /// draws it.
    #[test]
    fn no_variant_bets_on_a_font_having_a_glyph() {
        for variant in all_variants() {
            for frame in variant.frames() {
                if let Frame::Text(text) = frame {
                    assert!(
                        text.is_ascii(),
                        "{variant:?} draws {text:?} as text, which is a bet on the \
                         consumer's font; draw it as `Frame::Dots` instead"
                    );
                }
            }
        }
    }

    /// A blank frame reads as a dropped frame, not as part of the animation.
    #[test]
    fn no_frame_is_empty() {
        for variant in all_variants() {
            for frame in variant.frames() {
                match frame {
                    Frame::Text(text) => {
                        assert!(!text.is_empty(), "{variant:?} has an empty text frame")
                    }
                    Frame::Dots { cols, rows, mask } => {
                        assert_ne!(*mask, 0, "{variant:?} has a frame with no lit dots");
                        let cells = cols * rows;
                        assert!(
                            *mask < (1 << cells),
                            "{variant:?} lights a cell outside its {cols}x{rows} grid"
                        );
                    }
                }
            }
        }
    }

    /// Braille numbers its dots down the left column, then down the right,
    /// then the low pair — so the byte is not a row-major bitmap, and the
    /// spinner comes out mirrored if the translation is wrong.
    #[test]
    fn braille_bytes_map_to_the_cells_braille_names() {
        // Dot 1 is the top left; dot 8 is the bottom right; all eight is all
        // eight. Cell indices are row-major on the 2-wide, 4-tall grid.
        assert_eq!(braille(0b0000_0001), 1 << 0, "dot 1 is the top left");
        assert_eq!(braille(0b0000_1000), 1 << 1, "dot 4 is the top right");
        assert_eq!(braille(0b0100_0000), 1 << 6, "dot 7 is the bottom left");
        assert_eq!(braille(0b1000_0000), 1 << 7, "dot 8 is the bottom right");
        assert_eq!(braille(0xFF), 0b1111_1111, "every dot is every cell");
        assert_eq!(braille(0x00), 0, "no dots, no cells");
    }

    /// Each frame of the eight-frame spinner is the full cell minus one, and
    /// over the cycle every cell takes its turn — which is what makes it read
    /// as one dark dot going round rather than as flicker.
    #[test]
    fn the_braille_spinner_walks_one_dark_dot_around_the_ring() {
        let mut dark = Vec::new();
        for frame in BRAILLE_SPINNER {
            let Frame::Dots { mask, .. } = frame else {
                panic!("the braille spinner is drawn, not typed");
            };
            assert_eq!(
                mask.count_ones(),
                7,
                "a spinner frame lights {} dots, not seven",
                mask.count_ones()
            );
            dark.push((!mask) & 0xFF);
        }

        dark.sort_unstable();
        dark.dedup();
        assert_eq!(dark.len(), 8, "two frames leave the same dot dark");
    }

    /// A counter counts: 255 frames, each the next pattern, none of them
    /// blank.
    #[test]
    fn the_braille_counter_counts() {
        assert_eq!(BRAILLE_COUNTER.len(), 255);
        for (index, frame) in BRAILLE_COUNTER.iter().enumerate() {
            assert_eq!(*frame, braille_frame((index + 1) as u8));
        }
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
