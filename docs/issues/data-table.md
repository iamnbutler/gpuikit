# Data Table: sorting, filtering and selection over Table

## What it is

`Table` plus the interactions that turn a grid into a data view: sortable
columns, a filter, selectable rows.

## Why it survives triage — with a recommendation against a separate element

**The recommendation is to fold this into `Table` rather than ship a second
element.** shadcn separates them because its Table is unstyled markup and its
DataTable is a TanStack Table recipe; that split is an artifact of its
ecosystem, not a design. Primer ships one `DataTable`. A second `pub mod` here
would mean a second showcase page, a second `ELEMENT_COVERAGE` row and a
permanent question about which one to reach for.

This issue therefore exists to record the *decision*: build `Table` first with
these interactions in mind, and add them to `Table` behind explicit opt-ins.
If the sort/filter surface turns out to be large enough to be its own thing,
that is a discovery to make with `Table` in hand, not a structure to commit to
first.

## Prior art

- **Primer's DataTable** — one component, sorting as a column property.
- **TanStack Table** is the headless model everyone else wraps: state lives
  outside the component (sorting state, column visibility, row selection) and
  the component renders it. That separation is the part worth taking; a
  toolkit component should not own the sort order.
- Re-open both before implementing.

## What it has to close in this crate

- **Sorting is state the caller owns.** Take the sorted rows and the current
  sort descriptor; emit "the user clicked this header". Sorting inside the
  component means owning comparison for arbitrary cell types, which this crate
  should not do.
- **Selection.** Whether a header checkbox selects all is the first design
  question. `Checkbox` supports an indeterminate state already, which is what
  a partial selection needs.
- **Filtering is a `TextField` above the table**, not a table feature. Say so
  in the docs so nobody adds a filter input inside the element.
- **Pagination is rejected** in `docs/component-triage.md` until there is a
  paginated data source. If this lands and a consumer has one, that rejection's
  revisit trigger has fired.

## Accessibility

Roles needed: everything `table.md` needs, plus the sort direction on a sorted
`ColumnHeader` and the selected state on a `Row`.

## Blocked on

- `docs/issues/table.md` — **hard block**.
- `docs/issues/element-roles-convention.md`.

### Accessibility

`src/elements/sidebar.rs` is the only element in `src/elements/` that reports a
role — it had to, because its own issue's Accessibility section required one,
and it went ahead of the convention rather than shipping a landmark with no
role. It is the accidental worked example: read it, and note that `.role()`
lives on `StatefulInteractiveElement` and is reachable on any `div().id(…)`, so
a `RenderOnce` does not have to become a real `Element` to report one. Beyond
that file the crate's accessibility work is all in `src/markdown/`.

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
