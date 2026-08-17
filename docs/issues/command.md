# Command: a filterable action list in an overlay

## What it is

A single overlay that lists actions, filters them as you type, and runs the one
you pick. The pattern Zed calls the command palette and VS Code calls the same;
shadcn calls it Command, Headless UI does not ship it at all.

## Why it survives triage

It is the one entry from #59 that this toolkit's own consumers plausibly want
and cannot assemble from what exists. `Dialog` gives the overlay, `TextField`
gives the query field and `List` gives the rows, but the part that matters —
filtering, the selected-row model, and the keyboard contract that makes the
field and the list behave as one control — is exactly what is not there.

## Prior art

- **Zed's `picker` crate** is the closest thing to a reference implementation in
  gpui: a delegate trait supplies matches, and the picker owns the query field,
  the selection and the keyboard. Zed's command palette is one delegate over it.
  The delegate shape is the part worth stealing — it separates "what can be run"
  from "how it is shown", which is what lets one component serve a command
  palette, a file finder and a symbol jump.
- **shadcn's Command** wraps `cmdk`, which is a filtering primitive with a
  list, groups and an empty state. Its value is the empty/loading/group
  vocabulary rather than the matching.
- **Headless UI ships no equivalent**, which is a signal about scope: this is
  an application component, not a form primitive.

Re-open both before implementing; the citations above are by project and
component name deliberately.

## What it has to close in this crate

- **Matching is not the crate's business.** Take a `Vec` of candidates and a
  scoring closure, or take pre-matched rows. Do not embed a fuzzy matcher —
  every consumer already has opinions about ranking, and a mediocre built-in
  one is worse than none.
- **The overlay.** `Dialog` centres and scrims; a command palette usually wants
  to sit near the top of the window. Whether that is a `Dialog` option or a new
  placement depends on nothing outstanding — follow `docs/overlays.md`, which
  is the crate's overlay convention, and put the gap on `anchored().offset(…)`
  rather than on the anchored child.
- **The keyboard contract is the component.** Focus stays in the query field
  while up/down move the list selection; enter runs the selection; escape
  dismisses. `ContextMenu` already implements a version of this
  (`next_focus`/`selectable_indices` and their tests) and is the local
  precedent to follow.
- **An empty state.** `src/elements/empty.rs` exists; use it rather than
  inventing a "no results" row.

## Accessibility

Roles needed: the query field is a `TextField`; the results are a listbox with
options, and the field owns the selection (`aria-activedescendant`'s
equivalent). accesskit's `Role` enum has `ListBox` and `ListBoxOption`, so
there is no platform excuse. See the shared note below.

## Blocked on

- `docs/issues/element-roles-convention.md` — before it reports a role.
- Nothing else. The overlay question is settled: `docs/overlays.md` is the
  convention to follow when it places one.

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
