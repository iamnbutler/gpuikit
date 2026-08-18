# Slider keeps the drag after the pointer leaves the track (#177)

`Slider` never took a pointer capture at all: mouse-down, mouse-up and mouse-move were three
plain `div` listeners on its own track, and a `div` listener only fires while its hitbox is
hovered. A drag that left the track therefore lost the movement — the thumb froze at the edge
instead of pinning to the end of the range — and, if the button came up outside, lost the
release too, leaving `is_dragging` stuck `true` and the thumb wearing its dragging border. The
movement and the release now live on the **window**, registered via `Window::on_mouse_event`
from the `canvas()` paint closure that was already measuring the track — the pattern
`src/elements/splitter.rs` (#158) and `src/elements/input.rs` established, and the only hook
available, since `Window::on_mouse_event` fills the frame currently being painted and asserts
the paint phase, so it cannot be called from `render`. Only the press stays on the track div,
because a press always starts there. Both window handlers ignore anything but
`DispatchPhase::Bubble`, the up handler ignores non-`Left` buttons, and they are registered on
*every* paint rather than only while a drag is live — the frame that has to carry a drag was
painted before the mouse went down, so a conditional registration would arrive one frame late;
they guard on `is_dragging` internally instead. The move handler also ends the drag when
`!event.dragging()`, which covers the release the window never saw at all (the pointer left the
*window* with the button held, or another handler swallowed the mouse-up) and the case of the
slider being disabled mid-drag, since the window handlers are registered whether or not it is
enabled. `on_mouse_down` gains a `cx.notify()`: `set_value` is silent when the value does not
change, which is exactly what pressing on the thumb does, so without it the dragging border did
not appear until the value moved. No value maths changed — `value_from_position` already clamped
its percentage to `0..=1`, so pinning to the range end came for free once tracking survived the
leave. The public API is untouched.

`src/elements/slider.rs` is the only source file changed, plus a `### Fixed` entry under
`## [Unreleased]` in `CHANGELOG.md` (the section did not exist yet on this branch; it is added
after `### Changed`, matching 0.8.0's order, alongside the Breaking Changes / Added / Changed
entries already there from #187). Eight tests are added in `src/elements/slider.rs`: a `Harness`
view of the kind `src/elements/sidebar.rs` uses, holding an `Entity<Slider>` in a 200px-wide
wrapper offset 100px from the left of a 400px window, so a pointer can be outside the track and
still inside the window. Four regressions — a drag past the end pins to the maximum, a drag past
the start pins to the minimum, a release outside the track clears `is_dragging` and stops
tracking, and a move with no button held ends the drag without a later button-down move resuming
it. Three controls that pass before *and* after, so the regressions are not just re-asserting the
fix — a press moves the value and starts a drag, an in-track drag still tracks, a disabled slider
ignores the pointer. And one test pinning the `cx.notify()`, described below. Run against pre-fix `slider.rs` with this
test module appended, the four regressions fail — the pre-fix failure is a freeze rather than a
wrong number, the value simply staying where it was when the pointer left the track — and the
three controls pass.

## Review feedback

- **Required: pin the `cx.notify()` in `on_mouse_down`, or say at the line that it is
  deliberately untested.** Pinned — but not the way the feedback suggested, because that way does
  not work here, and finding out why is worth recording. A render counter on the `Harness` plus a
  press at the exact position of the current value passes with the `cx.notify()` deleted: gpui's
  `div` binds its active-state handlers unconditionally ("we unconditionally bind both the mouse
  up and mouse down active state handlers", `gpui/src/elements/div.rs`) and calls `window.refresh()`
  on any mouse-down over an id'd hitbox, so a simulated press draws a new frame either way. I built
  that test first and confirmed it was no pin before replacing it.
  `a_press_that_does_not_move_the_value_still_repaints` therefore calls `on_mouse_down` directly —
  same counter, same 50-in-0..=100 slider pressed at exactly its own value so `set_value` is a
  no-op, but no `div` in the path to repaint on its behalf. Verified as a pin: delete the
  `cx.notify()` and that test fails while the other seven pass. The line carries a comment naming
  the test and why it has to bypass the `div`. This makes eight tests rather than the spec's seven,
  a deviation in the direction the feedback asked for.
  One thing that follows and should be said plainly: because of that unconditional refresh, the
  symptom the spec attributes to the missing notify does not reproduce through a real press on
  gpui 1.15 — the border appears anyway, by accident of `div`'s internals rather than by anything
  `Slider` does. The notify stays because it makes the repaint the component's own guarantee, and
  it is now held by a test instead of by that accident.
- **Note: leave `value_from_position`'s hardcoded `thumb_radius = px(6.)` alone; it is being
  filed as its own issue.** Left untouched. The tests run at the default 16px rem, so they neither
  catch it nor depend on it being wrong; the test helper that maps a value to an x duplicates the
  same 6px inset and says so, rather than pretending the mapping is rem-derived.

## Directions from the orchestrator

- **Do not end the turn while the verification run is still going.** The dependency build was
  started in the background while the change was written, and every cargo run was joined before
  this file was finalised; the trailer below reports what the run actually
  returned, and its exit code was checked directly rather than through a pipeline.
- **`signal 9` under default parallelism means the linker was OOM-killed; `-j 1` is the answer,
  and never pipe the test command through anything that discards its exit status.** Recorded;
  what was actually run is in the trailer. No cargo invocation overlapped another.
- **The base has moved: check the `## [Unreleased]` section on this branch rather than trusting
  the spec.** Checked. It has Breaking Changes / Added / Changed from #187 and still no `### Fixed`
  as the spec said, so the new section was added after `### Changed` and nothing existing was
  replaced.

Verification: PASSED — `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test -j 1` (exit code 0; 459 lib tests and 2 doc tests passed, 0 failed)
