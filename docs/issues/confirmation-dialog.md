# Confirmation dialog: the destructive-action affordance Dialog is missing

## What it is

#59 called it Alert Dialog. `src/elements/dialog.rs` already has the shape — a
modal, a scrim, a title, a close button. What is missing is the *confirmation*
affordance: a question, a pair of actions where one is destructive, a sensible
default focus, and an announcement that reads as one thing rather than as a
window containing some text.

## Why it survives triage

This is the "partial" entry from #146's breakdown. `Dialog` covers half of it,
and the missing half is exactly the half that is easy to get wrong in a way
that costs a user their data.

## Prior art

- **Headless UI ships `Dialog` only** and treats the alert variant as a role
  and a focus decision on the same component. That is the right shape here too.
- **The WAI-ARIA alertdialog pattern**: the dialog is labelled by its title and
  described by its body, and initial focus goes to the *safe* action.
- Re-open both before implementing.

## What it has to close in this crate

- **A destructive button variant does not exist.** `src/elements/button.rs` has
  one variant, `Filled`, with its own `// todo: style through ButtonVariant`.
  A confirmation dialog whose destructive action looks identical to its cancel
  action is not worth shipping, so this is a real prerequisite and probably the
  first commit.
- **Initial focus.** On the non-destructive action, so that a reflexive
  return/space does the safe thing. `Dialog` needs to be able to say which
  child gets focus on open.
- **Title and description as a pair.** The dialog is labelled by one and
  described by the other; today they are just children.
- **Escape and outside-click.** For a destructive confirmation these should
  cancel, never confirm. Check what `Dialog` does today and make it explicit.
- **Build it as a `Dialog` mode, not a new element.** A second modal
  implementation is the outcome to avoid.

## Accessibility

Role needed: `AlertDialog`, which accesskit has, distinct from `Dialog` so that
assistive technology interrupts rather than queues.

## Blocked on

- A destructive `ButtonVariant`.
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
