# Gauge: a meter, standing apart from Progress on the role boundary

## What it is

A read-only display of a measurement within a range: linear (a bar with
optional min/current/max labels) and circular (an open arc with the current
value in the middle — SwiftUI's `accessoryCircular` shape) variants of one
element. `gauge(value, range)` with a required name, `.linear()` /
`.circular()`, `.current_value_label(…)`, `.min_label(…)`, `.max_label(…)`.

Traits: `ControlSized`. Not `Disableable` (a reading cannot be disabled — it
is not interactive), not `Labelable` (the name is *required*, so it is a
constructor argument; an optional label is a name an element forgets, which is
`src/a11y.rs` §2's whole argument), not `Clickable`/`Selectable`/`Orientable`.

## Why it survives triage

**The Table precedent is the right question and it gives the other answer.**
Table and Data Table became one module because their split was an ecosystem
artifact — "a second `pub mod` would have meant a permanent question about
which one to reach for," and sorting was just a property. Gauge/Progress is
not that split. ARIA separates `meter` from `progressbar`; accesskit separates
`Role::Meter` from `Role::ProgressIndicator`; SwiftUI separates `Gauge` from
`ProgressView` and documents the line: progress is task completion headed to
done, a gauge is a measurement that sits where it sits. Folding Gauge into
Progress puts two roles in one module and moves the "which one" question
*inside* the API, where it becomes a mis-announced screen-reader node instead
of a docs question. The house precedent this actually matches is `Splitter`:
when accesskit has the variant and the pattern has the name, the element
stands on it. One module, `gauge.rs`, `Role::Meter`, and both modules' docs
state the reach-for rule in one sentence each.

**And Gauge is buildable where Progress's a11y is stuck.** `src/a11y.rs`'s
`ELEMENTS_WITHOUT_A_ROLE` blocks `progress` on the indeterminate-progress
decision. A gauge has no indeterminate case — a measurement always has a value
— so it adopts its role on day one, and its adoption is the worked example the
`progress` row is waiting for.

Deliberately not built: an indeterminate gauge (contradiction in terms), a
threshold/color-zone API (SwiftUI's answer is `tint`, and here it is one
optional color argument; zones are a revisit when a consumer names one),
animation between values, and a vertical linear gauge (no consumer, no
`Orientable` until one appears).

## Prior art

- **SwiftUI, `Gauge`** — take the API surface whole: `value`, `in:` range,
  label, `currentValueLabel`, `minimumValueLabel`, `maximumValueLabel`, and
  the linear/circular style split. Also take its reason for existing next to
  `ProgressView` — that is this issue's central argument. Re-open before
  implementing.
- **WAI-ARIA, `meter` role** — the semantic contract: a scalar within a known
  range, not interactive, distinct from `progressbar`. Re-open for whether
  `aria-valuetext` guidance applies to the current-value label.

## What it has to close in this crate

- **`src/elements/progress.rs` is the anti-pattern to not copy.** 82 lines
  that name their own height (`px(8.0)`), take raw `Pixels` for width, report
  no role and implement no `ControlSized`. Gauge takes none of that shape. A
  follow-up should drag `Progress` up to whatever Gauge lands — same bar
  geometry off the rung — but that is Progress's issue, not this one.
- **The arc has no in-crate precedent, and this issue creates one.**
  `src/elements/loading_indicator.rs` is glyph frames — braille characters on
  a shared clock, not drawn arcs — so the crate has never painted an arc
  anywhere (no `PathBuilder` caller under `src/`). gpui ships
  `PathBuilder::arc_to`/`curve_to`; the circular variant paints two arcs
  (track and fill) in a `canvas`, the same paint-hook pattern
  `slider.rs`/`splitter.rs` already use for their canvases. This becomes the
  rendering precedent the crate cites next time something needs an arc.
- **Labels are children, not new machinery**: min/max flank the linear bar,
  the current value centers in the circular arc's opening, all sized off the
  rung's `text_size`.

## Accessibility

`Role::Meter` — accesskit's own variant, already in `role_requires_a_name`
(pinned: "a number with no name is a quantity of nothing") and deliberately
absent from `role_requires_keyboard_focus`: a meter is read, not operated, so
it declares `not_focusable` with that reason, which `Announce` records as a
decision rather than silence. One extension needed, named the way the triage
doc names gpui's missing `aria_sort`: `A11yValue::Number` in `src/a11y.rs`
requires all four of value/min/max/step, and **a gauge has no step**. The fix
is local and sanctioned by that module's own docs ("adding the field here"):
`step` becomes `Option<f64>`, with `Announce` skipping
`aria_numeric_value_step` when absent — which `Slider`, whose `step` is
already `Option<f32>`, will want on its own adoption anyway.

## Sizing

`ControlSized` off `ControlMetrics`, fixing exactly what Progress got wrong:
the linear bar's thickness derives from the rung (a stated fraction of `ink`,
in the element's file per `src/theme/control.rs`'s "what belongs here" note),
the circular variant's diameter is a stated multiple of the rung height, and
no dimension in `gauge.rs` is a bare pixel literal.

## Showcase

A `gauge` row in `ELEMENT_COVERAGE` and a page: linear with min/max labels,
circular with a centered current value, all three rungs, a custom tint — and,
beside them, a `Progress` bar with a caption stating the reach-for rule, so
the showcase teaches the boundary this issue draws.

## Non-goals

Folding into `progress.rs` or absorbing it; indeterminate state; threshold
zones; animation; vertical orientation; any interactivity — the moment a gauge
takes input it has become `Slider`, which already exists.
