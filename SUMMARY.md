# Take the showcase off the display-refresh treadmill

All three leads in the issue were real, and all three are fixed. The substantive
one is the loading indicator: gpui's `AnimationElement` calls
`window.request_animation_frame()` on every frame while its animation is
unfinished, and `request_animation_frame` is `on_next_frame(|_, cx|
cx.notify(current_view))` — so an `Animation::new(..).repeat()` re-arms a notify
of the *enclosing view* forever. In the showcase that enclosing view is the root
`Showcase`, so the Loading page was re-laying-out and repainting the whole
window, sidebar included, 60–120 times a second whether or not a glyph had
changed. `LoadingIndicator` now draws its frame from one process-wide
`LoadingClock` instead: a single driver task sleeps to the next boundary of
whichever registered frame period comes first — the union of the variants'
boundaries, not a quantized shared rate, because 10 Hz would turn Braille's
125 ms frames into a visible 100/200 ms limp — and then notifies exactly the
views showing an indicator. That is the same invalidation gpui was already
doing, at the rate the glyphs actually change: the Loading page's seven
indicators go from ~120 whole-window redraws a second to ~39, and a realistic
app with one spinner goes from 120 to 8. Subscribers expire after four of their
own frames without re-rendering, so an occluded or navigated-away window stops
paying; when the last one goes the driver returns and the clock costs nothing
until an indicator is rendered again. `LoadingIndicator::playing(bool)` is new
and purely additive — a paused indicator, like `App::reduce_motion()`, renders
frame 0 and subscribes to nothing. The private `LoadingFrame` helper and
`LoadingIndicatorVariant::animation_id` are gone, as is the 256-entry
`Vec<SharedString>` the extended braille variant allocated per render; the new
path allocates nothing per frame. One behaviour change: indicators share an
epoch, so one mounted mid-cycle starts at the shared timeline's current frame
rather than at frame 0 — two braille spinners on a page now spin in step.

The other two leads are done as filed. `Cargo.toml` gains
`[profile.dev.package."*"] opt-level = 2`, which optimizes dependencies (gpui
above all, compiled once and thereafter only linked) while `[profile.dev]`
itself stays at `opt-level = 0`, so iterating on this crate compiles exactly as
fast as before. The showcase's sidebar was rebuilt from scratch on every frame —
24 `format!`s, some seventy `SharedString`s and 48 boxed closures — to change
which single row was highlighted; the table is now a `NAV_SECTIONS` constant and
the rows are built once in `Showcase::new`, with `render` cloning them
(`ListEntry` is `Rc`-backed) and stamping `selected`. The Loading page keeps all
seven variants — a gallery should show them, and they now share one clock — and
gains a Pause/Play button, which isolates the indicators' cost from the page's
without navigating away. After this change, `grep -rn
"request_animation_frame\|on_next_frame\|refresh_windows\|window.refresh()" src
examples` returns only event-driven refreshes: nothing in the library or the
showcase asks for continuous frames. The regression guard that matters is
`indicators_do_not_request_a_frame_per_display_frame`, which mounts three
indicators and asserts `window.simulate_next_frame(cx) == 0` — reattaching a
repeating animation to any indicator fails it — paired with
`the_shared_clock_advances_over_time`, so "quiet" cannot be achieved by simply
not animating. **Not verified by running the app**: this box has no display, so
the felt improvement, and how much of it comes from the profile change rather
than the clock, is still unmeasured. The example-link OOM the issue mentions in
passing is untouched; it reproduces here only when several examples link
concurrently (`-j 1` links them all fine), and `[profile.dev] debug = 2` on
dependencies is the obvious lever, which the issue calls a separate problem.

Verification: PASSED — `cargo fmt --all -- --check` (clean); `cargo clippy --all-targets --all-features` (75 warnings, identical to the count on the unmodified tree — no new warnings); `cargo test --all-features -j 1` (424 lib tests + 2 doc tests passed, 0 failed, every example built and linked); `cargo test -j 1` (278 lib tests passed, 0 failed)
