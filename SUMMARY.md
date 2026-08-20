# Calendar: a month grid of selectable days (#157)

A minimal civil date in a new `src/date.rs` — `Date { year, month, day }`, private
fields behind a validating `Date::new`, Hinnant's `days_from_civil` /
`civil_from_days` underneath every operation so the leap-year rule appears once —
rather than a `chrono` / `time` / `jiff` dependency in this crate's public API.
That was the decision the issue said had to be settled before any grid is drawn,
and it is settled the way the issue recommended; the arithmetic is verified over
every day from 1800 to 2200 rather than sampled, and there is deliberately no
`Date::today`. On top of it, `src/elements/calendar.rs`: a six-by-seven month
grid, weekday headings, muted leading and trailing days so the grid never changes
height, single selection, a caller-supplied `today`, a disabled-day *predicate*,
month navigation, and the localisation parameters (`first_day_of_week`,
`month_labels`, `weekday_labels`). It is an entity because it holds three things
across frames a caller should not have to — the visible month, the
keyboard-focused day, and its `FocusHandle`.

The keyboard is bound **actions** in a `Calendar` key context registered by
`bind_calendar_keys` from `gpuikit::init`, next to `bind_select_keys`, not one
`on_key_down`: gpui dispatches bound actions before key-down listeners, so a raw
handler would hand arrows, Home/End, PageUp/PageDown and Enter to an enclosing
dialog. The visible month is symmetric — `set_visible_month` in,
`CalendarEvent::MonthChanged` out — so an owner can bring the grid to a date it
got from somewhere else. Accessibility goes through `src/a11y.rs`: `Role::Grid`
over `Role::Row` / `Role::ColumnHeader` / `Role::GridCell`, the grid as the one
tab stop, and `active_descendant` on the focused day, which is the *ancestor*
arrangement gpui honours rather than the sibling one it drops in silence — the
module docs say which of the two this is, because the distinction is invisible in
a diff. `examples/showcase.rs` gains a live page, `docs/component-triage.md` moves
Calendar from Issue to Shipped (13/5/11, with `EXPECTED` in `src/elements.rs` and
both prose restatements moved with it), and `CHANGELOG.md` records both modules.
Range selection is not built and not stubbed; the module documents what it would
change, since `selected: Option<Date>` would have to become an enum.

## Provenance

This spec was implemented once before, on the closed branch
`build/build_c5cb9fcbbfc8491d9b461797f7a2c087` (`94319c0`). Per the build
directions I started from that commit rather than from scratch: I cherry-picked
its `src/`, `docs/` and `examples/` changes onto current trunk (leaving its
`PROMPT.md` behind), re-read every count it touched against the trunk as it is
now, reviewed it, and changed what I disagreed with. The changes I made on top of
it are listed under "What I changed" below. I ran the suite here rather than
trusting its numbers.

## Review feedback

1. **Keyboard as actions, not one `on_key_down`.** Done — an `actions!` block,
   `CALENDAR_CONTEXT`, and `bind_calendar_keys(cx)` called from `crate::init`
   beside `bind_select_keys`, exactly as `select.rs` does it. The inherited
   commit had already followed this; I verified it against `select.rs` and added
   the test that proves the bindings are reachable in the real dispatch tree
   (item 3), which is the half a unit test cannot see.
2. **An in-direction for the visible month.** Done — `set_visible_month(date, cx)`
   pairs with `CalendarEvent::MonthChanged`, emits at most one event, and emits
   none when the month is already showing. Tested.
3. **Test the grid as a grid.** Done, and extended. `TestAppContext` tests cover:
   42 days always, starting on `first_day_of_week` and consecutive; paging from
   the 31st clamping to Feb 28 and emitting exactly one `MonthChanged`; a day
   cell announcing `Role::GridCell` with `selected` and `active_descendant` on
   the right two cells and exactly one claim per frame; `set_visible_month`
   saying so once; and a disabled day refusing selection. On top of those I added
   the keyboard test the feedback asked for, in a real drawn window
   (`VisualTestContext`, `crate::init`, the grid focused) so the keystrokes walk
   the dispatch tree: arrows by day and week, Home/End to the ends of that week,
   Enter and Space selecting, and `pagedown` from the 31st landing on Feb 28 with
   one `MonthChanged` — plus `shift-pagedown` and `pageup` showing that the
   focused day is kept and re-derived rather than paged back through
   `add_months`, which clamps and does not round-trip.

## Directions

- **Start from `94319c0` rather than from scratch.** Done, as described under
  Provenance. Cherry-picked the three trees, not the branch, so none of the
  `#190` / `#198` commits already in trunk came along.
- **Re-run the suite and re-read the counts against the trunk.** Done. The trunk
  still read 12 Shipped / 6 Issue / 11 Rejected in all three places, so the
  12→13 / 6→5 move applied cleanly and is correct as of this commit; I checked
  the `<!-- ratification -->` prose, the verdict table, the "Three things are
  checkable" paragraph and `EXPECTED` in `src/elements.rs` individually rather
  than trusting the patch.
- **Leave a seam for a date-picker field (#162).** The seam is the entity's
  public surface, and it is deliberately two-directional: `set_visible_month` /
  `visible_month()`, `set_selected` / `selection()`, `CalendarEvent::{Selected,
  MonthChanged}`, `Focusable` for the `FocusHandle` a popup owner needs, plus the
  builders (`today`, `disabled_days`, `first_day_of_week`, the label arrays) and
  `ControlSized` / `Disableable`. A date picker owns a `TextField` and an
  `Entity<Calendar>`, pushes what the user typed in with `set_visible_month` /
  `set_selected`, and reads the two events back out; nothing in this module reads
  a clock or a keymap it does not own, so the picker decides both. What it will
  *not* find is a way to make the calendar its own popup — placement is
  `docs/overlays.md`'s business and this module deliberately calls no
  `deferred(`.
- **Clippy is not mine.** Left alone; `.tasks/verify` is `cargo test --lib` and
  that is what I ran.

## What I changed on top of the inherited commit

- Added the keyboard test described above (feedback item 3's fourth assertion),
  with a `GridView` / `draw` harness modelled on `elements::select`'s.
- Marked the `docs/issues/calendar.md` hard-block on `date-picker.md`
  **Discharged** in `docs/component-triage.md`'s Prerequisites list, naming both
  `src/elements/calendar.rs` and the `src/date.rs` decision. The spec called for
  this and the inherited commit had not done it. The entry still names the issue
  path, which `every_written_issue_is_reachable_from_the_triage` requires.
- Added the `CHANGELOG.md` entries for both new modules, under `[Unreleased]` /
  `Added`. The inherited commit had none and every comparable change in that file
  has one.
- Fixed a misplaced doc comment in `examples/showcase.rs`: the new
  `render_calendar_page` had been inserted between `render_select_page` and its
  doc comment, silently re-attaching six lines about `Dropdown` and `Select` to
  the calendar. `render_select_page` has its docs back.
- Moved `pub mod date;` above `pub mod element_id;` in `src/lib.rs`, where the
  list is alphabetical, and fixed the `Calendar::new` doc example, which was
  missing the `anchor` argument the signature takes.

## Verification

- `cargo test --lib` — **535 passed, 0 failed**, which is trunk's 517 plus the 11
  `date::tests`, the 5 grid tests and the 2 keyboard tests. That is what
  `.tasks/verify` runs.
- `cargo check --example showcase --features examples` — clean.
- One warning in the tree, and it is not mine: the pre-existing `unused_mut` at
  `src/input/bindings.rs:461`. Clippy is untouched, per the directions.

## Known gaps

- No rendering test asserts pixels; `active_descendant` is applied at paint time
  behind gpui's `a11y.is_active()`, which no test here can switch on, so
  `day_a11y` is the declaration the tests hold rather than the painted node.
- `A11y` still models none of gpui's four grid-index fields
  (`aria_row_index` and friends). Reaching past the convention to gpui's builders
  would fail `no_element_calls_gpuis_a11y_builders_directly`, so each cell
  carries its full date as its name instead; adding the four fields to `A11y` is
  the follow-up. A disabled day likewise says so in its name, because gpui has no
  `aria_disabled`.
- Roving focus is still not a crate convention — this is its fourth caller after
  `Tabs`, `List` / `ContextMenu` and `Select`'s popup, and it copies `Select`
  rather than inventing a fifth mechanism.
