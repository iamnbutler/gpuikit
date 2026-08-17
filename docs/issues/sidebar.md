# Sidebar: a docked panel with a collapsed state

## What it is

A panel docked to an edge of a window that can be collapsed and restored.

## Why it survives triage, and why it is smaller than #59 imagined

shadcn's Sidebar is roughly twenty exported parts — provider, trigger, rail,
inset, header, footer, group, group label, group action, menu, menu item, menu
button, sub-menu, skeleton, and so on. That is a layout framework wearing a
component's name, and most of those parts are `List`, `Button` and `Separator`
with a prefix.

What is missing from this crate is smaller and real: **a docked panel** — an
edge, a width, a collapsed state, and a persistent-versus-overlay behaviour.
The showcase itself hand-rolls one (its own sidebar is a `div` with a border
and a `List`), which is the evidence that the panel is the missing piece and
the twenty parts are not.

## Prior art

- **Zed's `workspace::dock`** is the gpui-native reference and is the right
  scope: a dock position, a panel with a size, open/closed, and resizing.
  Read it first.
- **Primer** ships navigation as `NavList` plus layout primitives rather than
  as a Sidebar component — evidence that the contents of a sidebar are not the
  sidebar.
- Re-open both before implementing.

## What it has to close in this crate

- **The edge and the width**, with the width owned by the caller so it can be
  persisted.
- **The collapsed state**, and what collapsed means: fully hidden, or a narrow
  rail of icons. Pick one; the rail is the harder and more useful one, and it
  is what makes the component more than a `when(open, …)`.
- **Overlay versus push.** At narrow window widths a persistent sidebar has to
  become an overlay. That transition is the part a consumer cannot easily write
  itself.
- **Do not ship menu/group/header sub-components.** `List` (with
  `ListEntry::header`), `Separator` and `Button` already cover the contents.
  The showcase page should demonstrate exactly that composition, which is also
  the argument for the smaller scope.
- **Convert the showcase's own sidebar to it.** That is the acceptance test:
  if the component cannot express the sidebar this repository already draws, it
  is the wrong component.

## Accessibility

Role needed: `Complementary` for the region, with an expanded/collapsed state
on the trigger. accesskit has `Complementary`.

## Blocked on

- `docs/issues/resizable.md` — for the resizable edge only. Ship without it.
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
