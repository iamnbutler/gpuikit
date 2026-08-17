# Form: grouping and label association, not form state

## What it is

#59 described it as "state management wrapper around Field". This issue argues
for building considerably less than that.

## Why it survives triage — as something much smaller

**Form state does not belong in a UI toolkit.** Validation rules, submission,
async errors and dirty tracking are application concerns with as many opinions
as there are applications, and a toolkit that picks one makes itself unusable
by everyone who picked another. React's ecosystem has a dedicated library for
this precisely because the component library should not.

The evidence is worth stating plainly: **Headless UI ships `Fieldset` and
nothing else** for forms — grouping, a legend, and label association. That is
the whole of what a headless kit provides, and it is a deliberate line rather
than an omission.

What is genuinely missing here:

- **Grouping.** A fieldset with a legend, and a disabled state that cascades to
  everything inside it. Today each control takes `disabled` individually.
- **Label association.** `src/elements/field.rs` draws a label next to a
  control; nothing connects the two for an assistive technology, and clicking
  the label does not focus the control.
- **A shared error presentation.** `Field` has `error`; a group-level error
  summary has nowhere to go.

## Prior art

- **Headless UI's `Fieldset`/`Legend`/`Field`/`Label`/`Description`** — the
  exact scope being recommended. Read it before widening this issue.
- **Primer's form components** are a product's answer and include more; treat
  them as evidence about product need rather than about toolkit scope.
- Re-open both before implementing.

## What it has to close in this crate

- **Cascading disabled.** The mechanism matters: an ambient value read by the
  controls, not a prop threaded by hand. Get this wrong and every control needs
  a `disabled` argument at every call site, which is what happens today.
- **Label-to-control association**, which needs
  `docs/issues/element-roles-convention.md` to have decided how an element is
  named. Clicking a label focusing its control is the visible half of the same
  fix.
- **Do not add submission, validation or dirty tracking.** If this issue grows
  those, it has become the thing this argument is against.

## Accessibility

Roles needed: a labelled group for the fieldset, and a name relationship from
label to control on every `Field`.

## Blocked on

- `docs/issues/element-roles-convention.md` — the naming half is the whole
  point.

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
