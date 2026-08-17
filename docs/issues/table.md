# Table: rows, columns, and a header that stays put

## What it is

A grid of cells with a header row, column alignment, and a body that scrolls
under a header that does not.

## Why it survives triage

It is the only entry on #59's "Future — Data & Complex" list that is a genuine
primitive rather than a composition of one. `markdown` already renders tables
(`src/markdown/elements/`), which is proof the layout is achievable and *not* a
reason to skip it: that renderer is coupled to markdown events and has no
column model, no sorting hook and no selection.

## Prior art

- **Primer's DataTable** is the closest match to what a product actually needs,
  and its column definition shape — a field, a header, an optional renderer, an
  alignment — is worth copying. Primer's roster is evidence about product need
  rather than about what a headless kit owes everyone, which is the right kind
  of evidence for this component.
- **Zed's `ui::data_table`** is the gpui-native reference. Read it before
  designing the column API.
- **Headless UI ships no table**, which is why this issue stops at the
  primitive and hands sorting, filtering and paging to `data-table.md`.

## What it has to close in this crate

- **A column model.** Width (fixed, flexible, content), horizontal alignment,
  and a per-cell render closure. This is the whole API surface worth arguing
  about; everything else follows from it.
- **A sticky header.** The header stays while the body scrolls. `ScrollArea`
  exists; whether the header lives outside it or is drawn sticky inside is the
  layout decision to make.
- **Row virtualisation is out of scope here.** `src/elements/list.rs` exists;
  if `Table` needs it later, it should reuse that rather than grow its own.
- **Selection is out of scope here.** Row selection belongs with sorting and
  filtering in `data-table.md`.
- Cell text must wrap. The markdown renderer had to fix exactly this: a flex
  item's automatic minimum size is one unbroken line, so a long cell runs off
  the edge instead of wrapping. Do not rediscover it.

## Accessibility

Roles needed: `Grid` (or `Table`), `Row`, `ColumnHeader`, `Cell`, with row and
column counts reported. accesskit has all of them.

## Blocks

`docs/issues/data-table.md` — **hard block**. Do not build them together; a
table whose first consumer is a sorting API will grow the sorting into its
layout.

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
