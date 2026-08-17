# Calendar: a month grid of selectable days

## What it is

A month grid: weekday headers, the days of the month, selection, and navigation
between months.

## Why it survives triage

It cannot be composed from anything the crate has, and it is the prerequisite
for `date-picker.md`. It is also the entry with the largest gap between "looks
simple" and "is simple".

## The decision that has to be made first: what is a date?

This crate has **no date type and no date dependency**. `Cargo.toml` has no
`chrono`, no `time`, no `jiff`. That is the real content of this issue, and it
should be settled before any grid is drawn:

1. **Take a dependency** (`chrono`, `time` or `jiff`) and use its date type in
   the public API. Simplest to build; puts a date library in the public surface
   of every consumer, which is a real cost — see how pulldown-cmark's presence
   in this crate's public API forced a coordinated bump.
2. **Define a minimal `Date { year, month, day }`** in the crate and convert at
   the boundary. Keeps the surface clean; means writing calendar arithmetic
   (leap years, month lengths, weekday-of-date), which is small, well-specified
   and easy to unit-test exhaustively.
3. **Be generic over a `Date` trait** the consumer implements. Most flexible,
   worst ergonomics, and the hardest to keep honest.

**The recommendation is (2)**, with the arithmetic unit-tested against known
values, precisely because a public dependency here is expensive and permanent
and the arithmetic is not.

Localisation — first day of the week, weekday names, month names — is part of
the same decision. Take them as parameters with English defaults rather than
pulling in an i18n stack.

## Prior art

- **Headless UI ships no calendar**, which is worth noting: it is the shape of
  component that is easy to get subtly wrong and hard to make accessible.
- **The WAI-ARIA date grid pattern** is the keyboard contract: arrows move by
  day, page up/down by month, home/end to the ends of the week.
- Re-open both before implementing.

## What it has to close in this crate

- **The grid itself.** Six rows of seven, with leading and trailing days from
  the adjacent months rendered differently, so the grid does not change height
  between months.
- **Selection modes.** Single, and range. Range is where most of the complexity
  is (hover preview of the range being drawn); ship single first.
- **Disabled days.** A predicate, not a list.
- **Today.** Marked distinctly from selected, and the "what is today" value
  passed in rather than read from the clock — the crate's tests cannot depend
  on the current date.

## Accessibility

Roles needed: `Grid` with `Row` and `Cell`, the selected day reported as
selected, and the focused day moved by the keyboard rather than by tabbing
through 30 cells.

## Blocks

`docs/issues/date-picker.md` — **hard block**, including the date-type decision.

### Accessibility

No element in `src/elements/` reports a role today — `grep -rn '\.role(' src/elements/`
returns nothing, and the crate's only accessibility work is in `src/markdown/`.
gpui builds an `accesskit` node for an element that has *both* an id and an
`Element::a11y_role`, and it hashes the element's whole id path into the node
id, so a duplicate id is a `debug_assert!` in debug and a silently missing node
in release. Read `src/element_id.rs` before adding a role to anything.

**This component must not invent its own mechanism.** `docs/issues/element-roles-convention.md`
decides once how an element reports a role, a name and its state; that should
land first. The roles this component needs are named below so the convention
issue can be checked against them.

### Sizing

The shared control size scale exists as of the #141 change: `ControlSize` on
`gpuikit::theme::control`, resolved through `Themeable::control` into a
`ControlMetrics` (height, `padding_x`, `gap`, `radius`, `text_size`,
`line_height`, `ink`), taken by a control through
`gpuikit::traits::control_sized::ControlSized`. This component must implement
`ControlSized` and take every dimension from the rung rather than naming one.
Anything genuinely specific to this component's shape stays in its own file,
keyed off the rung — see the "What belongs here" note at the top of
`src/theme/control.rs`.

### Showcase

`src/elements.rs` has a `showcase_coverage` test that fails the build when a
`pub mod` has no row in `examples/showcase.rs`'s `ELEMENT_COVERAGE`, and a
second that fails when a row names a page no match arm renders. A showcase page
is therefore a build requirement, not a convention: this component does not
compile into the crate until it is visible in the showcase.
