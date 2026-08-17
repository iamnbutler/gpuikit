# gpuikit Component Status

## Implemented

- Accordion
- Alert
- Aspect Ratio
- Avatar
- Badge
- Breadcrumb
- Button
- Button Group
- Card
- Checkbox
- Collapsible
- Context Menu
- Dialog
- Dropdown Menu
- Empty
- Field
- Icon Button
- Input
- Kbd
- Label
- List
- Loading Indicator
- Popover
- Progress
- Radio Group
- Scroll Area
- Select
- Separator
- Sidebar
- Slider
- Switch
- Table (sorting, filtering and selection are one element — see
  [docs/component-triage.md](docs/component-triage.md))
- Tabs
- Text Field
- Textarea
- Toast
- Toggle
- Toggle Group
- Tooltip
- Typography

## Removed — disabled pending performance work

Both were removed in [#121](https://github.com/iamnbutler/gpuikit/pull/121) and
shipped removed in 0.5.0. They are listed here rather than deleted so that
whoever revives one does not rebuild the same problem.

- **Skeleton** — the pulse ran through `Animation::new(1500ms).repeat()`. A gpui
  `AnimationElement` asks for another frame for as long as its animation is
  live, and `.repeat()` is never done, so a single skeleton pinned its window at
  the display refresh rate and everything else on that window re-laid-out and
  repainted at that rate too. (`Skeleton::animated(false)` did not help: the
  animation was attached unconditionally and the callback merely returned the
  element unchanged, so the repaint loop ran with the pulse "off".) To bring it
  back it has to stop requesting frames — drive the pulse off a timer or one
  shared animation entity — not merely animate more cheaply. That timer now
  exists: `src/elements/loading_indicator.rs` has a process-wide clock that
  wakes only when a frame actually changes and notifies only the views showing
  one. A revived Skeleton should subscribe to it rather than reach for
  `with_animation`.
- **Grain** — paints one quad per 4px cell inside a `canvas`, roughly 60k quads
  for a 1200×800 area. Tolerable on an idle window; it was what made the
  skeleton's forced repaint read as "a ton of lag" in the old single-scroll
  showcase, where several of each were mounted at once. Needs to become a
  shader or a tiled texture before it comes back.

## Not Yet Implemented

Every component that was on the old "Deferred" and "Future — Data & Complex"
lists has been re-triaged against the crate as it stands: see
[docs/component-triage.md](docs/component-triage.md) for a decision per
component. Eleven have shipped — eight already had when the triage was taken,
`Table` and `Data Table` have since been built as one module, and `Sidebar` has
shipped as `src/elements/sidebar.rs` — eleven are rejected with a reason and a
named revisit trigger, and seven have a
ready-to-file issue body under `docs/issues/` — along with three prerequisites
the triage surfaced (an element role convention, a menu-vs-listbox naming
decision, and adopt-or-delete for `src/traits/portal.rs`). Two are settled: the
portal trait is deleted and the convention that replaces it is
[docs/overlays.md](docs/overlays.md), and the role convention is `src/a11y.rs`
— an element implements `traits::accessible::Accessible` and applies the `A11y`
it returns with `.announce(a11y)`, with `Button` as the worked example. `Table`
still reports no accessibility roles, but now for its own reason rather than a
missing decision: its cells need derived ids before they can carry one. Only
the menu-vs-listbox naming decision is still open.

The lists that used to live here are gone deliberately. Restating a roster in a
second, uncheckable place is how the old one stayed alive for a year after the
reasons behind it expired.
