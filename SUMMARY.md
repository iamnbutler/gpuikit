# Calendar — a month grid of selectable days (#157)

This branch ships **one of the four approved specs in full**: Calendar (#157),
together with the date type the issue said had to be settled before any grid is
drawn. Command (#159), Combobox (#160) and Form (#164) are **not implemented** —
see "What is not here, and why" below, which is the most important section of
this document.

`src/date.rs` is a minimal civil `Date { year, month, day }` with private fields
and a validating constructor, plus `Weekday`. It is pure `std` and imports no
gpui. The arithmetic is Howard Hinnant's `days_from_civil` / `civil_from_days`
with one leap-year rule, so `weekday`, `add_days`, `add_months` and
`is_same_month` are all defined through `to_days` / `from_days` rather than each
carrying their own calendar knowledge; `round_trips_every_day_from_1800_to_2200`
walks ~146,000 consecutive days rather than sampling. There is deliberately no
`Date::today` — a UI toolkit that reads the system clock cannot be tested and
cannot be told about the user's zone, so `Calendar` is *given* its `today`.
`chrono` / `time` / `jiff` stay out of the public API, which is what the issue
recommended.

`src/elements/calendar.rs` is a six-by-seven grid — always 42 days, so the height
does not change as the user pages through the year — with weekday headings, muted
leading and trailing days, single selection, a disabled-day *predicate*, month
navigation, and localisation parameters for month and weekday names. It is an
entity because it holds three things across frames a caller should not have to:
the visible month, the keyboard-focused day, and its `FocusHandle`. The keyboard
is **actions in a `Calendar` key context**, bound by `bind_calendar_keys` which
`crate::init` calls next to `bind_select_keys` — not one `on_key_down`. It
announces `Role::Grid` (named by the visible month) over `Role::Row`,
`Role::ColumnHeader` and `Role::GridCell`, with the grid as the one tab stop and
`active_descendant` on the focused day; the module docs say in as many words that
this is the *ancestor* arrangement gpui honours, as distinct from the sibling
arrangement it drops in silence. `docs/component-triage.md` moves Calendar from
Issue to Shipped, with `EXPECTED` in `src/elements.rs`, the two prose
restatements and the attribution section's counts all moved in the same commit
(12/6/11 → 13/5/11).

**Verification.** `cargo test --lib` — **533 passed, 0 failed** (514 on the base,
plus 11 `date::tests` and 5 `calendar::tests` and the coverage tests re-run).
`cargo check --lib` is clean. `cargo check --example showcase --features examples`
is clean. The only warning anywhere is the pre-existing `unused_mut` at
`src/input/bindings.rs:461`, which is untouched. The base's `.tasks/verify` runs
`cargo test --lib`, which is the suite reported above.

## What is not here, and why

I had 60 minutes of wall clock for four component specs — roughly 2,500 lines of
new code against an unfamiliar API — and the repository has no `target/`, so the
cold `gpui` compile consumed the first ~13 minutes before any of it could be
type-checked. I implemented the specs in the order that produced the most
*complete, verified* work rather than the most partial work: Calendar first,
because its spec was the one whose scout had actually run `cargo test --lib` and
`cargo check --example showcase --features examples`, and because `date.rs` is
pure `std` and therefore the one piece that could be written with high confidence
without a compiler.

Command (#159), Combobox (#160) and Form (#164) are **untouched**. Nothing about
them is half-landed: no stub modules, no coverage rows pointing at pages that do
not exist, no triage rows moved in advance. This matters because every one of
those specs is gated by the same four coverage test modules — a `pub mod` with no
showcase page, or a triage row moved without its three restated counts, fails the
build in several places at once. A partial component here is not a partial
success; it is a red suite. Each of the three remains exactly as approved and can
be picked up against this trunk unchanged, and the two seams they need are now
in place: `Calendar::set_visible_month` (below) and a second worked example of
the actions-not-`on_key_down` rule.

## Review feedback

### Spec 3 of 4 — Calendar (#157), the spec that shipped

1. **Keyboard must be actions, not one `on_key_down`.** Done, and this is a
   change from the spec, which said "one `on_key_down` on the grid". There is an
   `actions!` block of eleven actions, a `CALENDAR_CONTEXT` key context on the
   grid, and a public `bind_calendar_keys` called from `crate::init` beside
   `bind_select_keys`, with the ordering reasoning in the comment there. No raw
   key handler anywhere in the module — `select.rs`'s type-ahead exception does
   not cover arrows or Enter, and none of these keys is a letter.
2. **An in-direction for the visible month.** Done:
   `Calendar::set_visible_month(date, cx)` pairs with the `MonthChanged` event it
   already emitted, and both go through one private `show_month`, so the event
   fires at most once and not at all when the month is already showing. There is
   a test for exactly that. `set_selected` moves the month too, for the same
   reason. This is #162's single stated requirement on this component.
3. **Build the harness and write the three assertions.** Done, as five
   `#[gpui::test]`s over a `TestAppContext` entity: the grid is always 42 days
   starting on `first_day_of_week` (checked for both Sunday- and Monday-first);
   paging from Jan 31 lands on the clamped Feb 28 and emits **exactly one**
   `MonthChanged`; a day cell announces `Role::GridCell` with `selected` and
   `active_descendant` on the right two cells, and exactly one cell in the grid
   claims the descendant; the month can be set from outside and says so once; a
   disabled day cannot be chosen. The module docs carry the sentence you asked
   for about which of the two `active_descendant` arrangements this is.

### Spec 1 of 4 — Combobox (#160)

1. **The draft in "what is already in the working tree" does not reach me.**
   Understood and acted on: I treated that section as design prose and started
   from `main`. Nothing was inherited and nothing was searched for. **Not
   implemented** for time, as above.
2. **Route (b) is refuted; settle the keyboard first.** Not reached. I did not
   spend the budget establishing which of the two remaining routes works, because
   I could not have built on the answer in the time left. Recording what I did
   verify, so the next Builder is not starting cold: `KeyBinding::new(key, action,
   Some(context))` with a plain string context is the only form used anywhere in
   this crate today (`select.rs`, `dialog.rs`, `a11y.rs`, `input/bindings.rs`) —
   the `"Parent > Child"` predicate form the reviewer confirmed exists in
   `gpui/src/keymap/context.rs` is *unused here*, so its first use in this
   repository is still unproven at runtime even though the parser supports it.
3. **Coverage entry must not point at another element's page.** Agreed and
   uncontested; there is no combobox coverage row at all, rather than one aimed
   at the select page.
4. **It compiles in this repository — do the lift, get `select.rs` green, then
   build.** Confirmed independently: a cold `cargo test --lib` completed here and
   `select.rs`'s suite is green on this branch. `select.rs` is **unmodified**, so
   the lift is still a clean starting point.
5. **`examples/showcase.rs` is shared with #165.** Honoured. My edits to
   `ELEMENT_COVERAGE` and `NAV_SECTIONS` are single inserted lines in
   alphabetical / sectional position, and the `Showcase::render` match gains one
   arm beside the `select` arm. Nothing was reordered or reformatted.

### Spec 2 of 4 — Command (#159)

1. **Drop the `active_descendant` claim.** Not implemented, so not claimed. It is
   recorded in `calendar.rs`'s module docs which arrangement is which, and that
   both the combobox and the command palette are in the sibling case and must
   decline it — so the decision is written down in the trunk even though neither
   component is here yet.
2. **Binding-depth citation.** Not reached; no `bind_command_keys` exists to
   carry the comment. The citation is preserved in this document instead:
   `KeyBindingContextPredicate::Descendant` at `gpui/src/keymap/context.rs:181`,
   parsed from `>` at `:361`, and `Keymap::bindings_for_input` at
   `gpui/src/keymap.rs:173` sorting `depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))`.
3. **Three runs, in order.** `cargo test --lib` and
   `cargo check --example showcase --features examples` were both run and are
   reported above. The real-window keyboard test was not written, because the
   component it would test is not here.
4. **The triage move is four edits or none.** Agreed, and applied to *Calendar*
   rather than Command: the row, `EXPECTED`, both prose restatements and the
   attribution counts moved in one commit, and `docs/issues/calendar.md` is still
   reachable from the triage's Prerequisites section so
   `every_written_issue_is_reachable_from_the_triage` still passes. Command's row
   is untouched at `Issue`.

### Spec 4 of 4 — Form (#164)

1. **Adopt one control onto the context.** Not implemented. If it is picked up
   from here, the reviewer's instruction stands and `Checkbox` is still the
   cheapest candidate; `field.rs` is unmodified, so `field()` is still
   argument-less and the breaking change has not been taken half-way.
2. **State the deferred-draw hazard narrowly.** Not reached. Recording the
   correction so it is not lost: the rule is *read the ambient value in `render`
   and pass it into whatever you defer; never call `disabled_here()` inside a
   deferred closure* — not "deferred elements are broken".
3. **"A bounded leak" is an assumption about callers.** Not reached, and the
   correction likewise recorded: the registry is bounded by the number of
   distinct field ids *ever rendered*, which is unbounded for any caller deriving
   a field id from a row or a record.
4. **Compile the showcase.** Done —
   `cargo check --example showcase --features examples` is clean on this branch,
   for the page this change actually adds.

## Directions for this implementation

- **Four separable commits, integrating rather than paralleling.** One commit,
  because one spec shipped. The showcase edit reads the existing structure: the
  calendar page is a `render_calendar_page` beside `render_select_page`, its nav
  entry is in the existing Input section next to Select, and its coverage row is
  in alphabetical position — no parallel structure was appended.
- **Base is another build's branch (#190, #198), not main.** The clone was
  actually checked out at `3973146` (the #201 merge), which does **not** contain
  `.tasks/verify`. I reset the branch onto `ec4c559` — the tip of
  `build/build_f59c901a6415454fb88252baade54fbb`, which carries #190 and #198 —
  before making any commit, so the branch really does have the declared base and
  the verify script in it. This is worth flagging: the environment and the
  direction disagreed, and I followed the direction.
- **Read the verify script.** Done. It is `exec cargo test --lib "$@"`, and the
  comment explains why it is not `--all-features` (eight gpui links, the OOM of
  #180) and not bare `cargo test` (doctests link ~71 binaries). That is the suite
  reported above.
- **Measured facts: 514 tests, 30 pre-existing clippy warnings.** Taken as given.
  I ran no clippy and cleaned up none of its noise; #203 owns that.
- **#162 (Date Picker) was sent back; leave the seam and describe it.** No
  date-picker field was built. The seam is: `Calendar` is an `Entity` whose whole
  public surface is `Calendar::new(id, anchor, cx)` plus the builders
  (`selected`, `today`, `disabled_days`, `first_day_of_week`, `month_labels`,
  `weekday_labels`, `Disableable`, `ControlSized`), the readers `selection()`,
  `visible_month()`, `days()`, `is_day_disabled()`, the setters `set_selected()`
  and `set_visible_month()`, and `EventEmitter<CalendarEvent>` with
  `Selected(Date)` and `MonthChanged(Date)`. A date picker owns a `TextField` and
  an `Entity<Calendar>`, subscribes to `Selected` to write the field, and calls
  `set_visible_month` when the user types a parseable date — both directions
  exist and neither requires touching this module. It will need the same popup
  and keyboard answer the combobox needs, which is the open question named above.
- **The keyboard rule applies beyond the spec it is attached to.** Followed for
  Calendar, which is the component here: actions and a key context, no raw
  `on_key_down`, no exception claimed. The rule is restated in `calendar.rs`'s
  module docs with the `docs/menus-and-listboxes.md` §3 citation, so the next
  component in this neighbourhood meets the argument in a second place.

## Known gaps in what did ship

- Nothing draws a *range*. `selected` is `Option<Date>` and the module documents
  what range selection would break rather than stubbing it.
- `A11y` still has no `aria_row_index` / `aria_column_index` /
  `aria_row_count` / `aria_column_count`, so a day cell carries its full ISO date
  as its accessible name instead of a grid position. Reaching for gpui's builders
  directly would fail `no_element_calls_gpuis_a11y_builders_directly`; adding the
  four fields to `A11y` is the follow-up.
- gpui has no `aria_disabled`, so a disabled day is distinguishable only by the
  `Click` action its node does not offer.
- The calendar's rendering is exercised by unit tests over `days()`, `day_a11y()`
  and the entity's events, not by a real-window keyboard test that presses
  `pagedown` and reads the grid back. The bindings themselves are therefore
  declared-and-registered but not observed firing.
