# Combobox: a text field that filters a list of choices

## What it is

A text input that also offers a list of values, filtered by what has been
typed, where the typed text may or may not be constrained to the list. Distinct
from `Select` (choose from a fixed list, no typing) and from Command (run an
action, not choose a value).

## Why it survives triage

`Select` cannot be typed into, and a `TextField` beside a `Dropdown` is two
controls that do not share a selection. The combobox is the only entry on #59's
list that fills a real gap between two things the crate already ships.

## Prior art

- **Headless UI's Combobox** is the reference for the behaviour contract, and
  it is worth following closely: it distinguishes the input value from the
  selected value, handles the "typed text is not in the list" case explicitly,
  and supports multiple selection. It is one of the few components Headless UI
  ships at all, which is a strong signal it is a primitive rather than a
  convenience.
- **shadcn's Combobox** is a `Popover` containing a `Command`, which is a
  useful reminder that this and `command.md` share machinery — but it is a
  composition recipe, not a component.
- Re-open both before implementing.

## What it has to close in this crate

- **The naming and layering problem, first.** `src/elements/select.rs` imports
  `DropdownMenu` and `DropdownOption` from `src/elements/dropdown.rs` — one
  component is built on the other's internals — and `ELEMENT_COVERAGE` maps both
  modules to the same showcase page. A combobox is a third thing in that
  neighbourhood and adding it before the naming is settled guarantees a fourth
  overlapping popup implementation. This is a hard block, not a preference.
- **Value versus text.** The state has to hold both, and say what happens on
  blur with unmatched text: revert, keep, or create. Pick one default and make
  the others explicit options.
- **Filtering is the caller's.** Same argument as `command.md`: take the
  filtered options, or take a predicate.
- **Reuse `TextField`.** It already owns the border, background, focus and
  disabled states and takes adornments — the chevron is a suffix adornment.

## Accessibility

Roles needed: `ComboBox` on the field, `ListBox`/`ListBoxOption` on the popup,
with the expanded state and the active option reported. accesskit has all of
them.

## Blocked on

- `docs/issues/menu-vs-listbox-naming.md` — **hard block**.
- `docs/issues/element-roles-convention.md`.
- Nothing else. Its popup follows `docs/overlays.md`. Note that matching the
  trigger's width — which a combobox wants — is the one thing that document
  names as unbuilt, and as the trigger that would reopen the abstraction
  question; this component will want it first.

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
