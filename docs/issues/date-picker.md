# Date Picker: a field that opens a calendar

## What it is

A text field showing a date, with a calendar in a popup for choosing one.

## Why it survives triage

It is the composition that makes `calendar.md` useful in a form, and the
composition is not trivial: parsing typed dates, formatting the selection, and
keeping the field and the grid in agreement are all real work.

## Prior art

- **shadcn's Date Picker** is explicitly a recipe — a `Popover` containing a
  `Calendar` — rather than a component, which is a fair description of most of
  the value here.
- **The WAI-ARIA combobox-with-grid pattern** is the closest specified
  behaviour for a typable date field with a popup grid.
- Re-open both before implementing.

## What it has to close in this crate

- **Typing.** A date field that cannot be typed into is worse than a text box
  for anyone entering a date they already know. That means parsing, which means
  deciding what formats are accepted, which is a localisation question — take a
  parse and a format closure rather than guessing.
- **The popup.** A `Calendar` anchored to the field. This is the third anchored
  overlay in this triage's surviving set; do not hand-roll a seventh
  `anchored()`/`deferred()` pair.
- **Reuse `TextField`.** The calendar-opening button is a suffix adornment; the
  field owns the border, focus and disabled states already.
- **Agreement.** Typing a valid date moves the grid; picking in the grid
  rewrites the field. Say what happens to invalid typed text on blur.

## Accessibility

Roles needed: `DateInput` on the field, plus everything `calendar.md` needs on
the popup, with the expanded state reported. accesskit has `DateInput`.

## Blocked on

- `docs/issues/calendar.md` — **hard block**, and the date-type decision inside
  it especially: this component's whole public API is dates.
- Nothing else for the popup: it follows `docs/overlays.md`.
- `docs/issues/element-roles-convention.md`.

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
