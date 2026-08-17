# Resizable: a draggable splitter between two panes

## What it is

Two panes with a draggable divider between them. **Not** a general pane tree —
see below.

## Why it survives triage, and why it is smaller than #59 imagined

shadcn's Resizable is a wrapper over a panel-group library with nested groups,
persisted layouts and collapsible panels. Zed has a real pane tree
(`workspace::pane_group`), and it is a workspace-level structure with splitting,
joining and serialisation — far more than an element.

The part that is genuinely a toolkit element is the **splitter**: one divider,
two neighbours, a drag that moves the boundary, a minimum size on each side, and
a keyboard equivalent. Everything above that is an application's layout model.
Build the splitter; do not build the tree.

## Prior art

- **Zed's `workspace::pane_group`** — read it to see what is deliberately *not*
  being built here.
- **The WAI-ARIA separator pattern** is the reference for the keyboard
  contract: a focusable divider that arrow keys move, with home/end going to
  the extremes.
- Re-open both before implementing.

## What it has to close in this crate

- **Where the split ratio lives.** The caller's, almost certainly — a component
  that owns its own layout cannot have it persisted or reset. Emit the new
  ratio; take the current one.
- **Minimum sizes.** A splitter without them drags a pane to zero and strands
  its content. Both sides need a floor, and the drag has to clamp rather than
  overshoot and snap back.
- **Hit area versus visual width.** A 1px divider is unhittable. The drawn line
  and the interactive band are different sizes; this is the detail that decides
  whether it feels good.
- **The cursor.** gpui has resize cursors; set the right one on the band.
- `src/elements/separator.rs` is the visual half of this and should be reused
  for the line rather than redrawn.

## Accessibility

Role needed: `Splitter`, with its current, minimum and maximum position
reported and arrow keys moving it. accesskit has `Splitter`.

## Blocks

The resizable edge of `docs/issues/sidebar.md`. Sidebar can ship without it.

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
