# Form: grouping and label association, not form state

Adds `src/elements/form.rs`: a `Fieldset` with a legend, a description, a
group-level error, and a `disabled` that cascades to everything inside it
through an **ambient** `FormContext` rather than a prop threaded by hand.
`Field` adopts that context — it takes a required id, announces a named
`Role::Group`, publishes its label as the ambient accessible name and the focus
handle its label click lands on, and inherits an enclosing fieldset's
`disabled`. `Checkbox` becomes the first leaf control to *read* the context, so
the mechanism ships with a consumer: a `Checkbox` inside a disabled `Fieldset`
is disabled without anything between them saying so, and a click on its
`Field`'s label lands focus on it. `Field` leaving
`a11y::ELEMENTS_WITHOUT_A_ROLE` is the note that list carried for it, discharged
— by a different route than it assumed, since gpui has no `labelled_by`
builder, so association is expressed as the *result* of the relation rather
than the relation itself.

There is no submission, no validation, no dirty tracking and no form-level value
store, and nothing here pushed toward one: the two things a control genuinely
cannot work out for itself are what group it is in and what it is called, and
both are answered without any state to own. `WithFormContext` is the one
hand-written `gpui::Element` under `src/elements/` — the scope has to be open
across `request_layout`, `prepaint` and `paint`, three moments a `RenderOnce`
cannot get between — and it reports no id and no role, so wrapping a subtree in
it changes neither the accessibility tree nor the element-id path. The scope
itself is a thread-local stack with a `Drop` guard, so a child that panics
mid-layout cannot leave it one deep; a `gpui::Global` was rejected because
reading one needs `&mut App`, which would put a `cx` argument on `disabled_here`
and take the ambient value straight back to being threaded by hand.
`cargo test --lib`: **530 passed, 0 failed** (517 on trunk plus 13 new), and
`cargo check --example showcase --features examples` is clean.

`field()` is a breaking change: it is `field(id)` now, and `Default` is gone.
Every call site in the repo is updated; downstream consumers are not.

## Review feedback

1. **Adopt one control, with an end-to-end test.** Done, mostly — `Checkbox`
   reads `form::disabled_here` and `form::focus_handle_here`, and two drawn-window
   tests in `form.rs` prove both: a checkbox in a disabled `Fieldset` (where
   neither it nor its `Field` says `disabled`) does not toggle when clicked, and
   a click on a `Field`'s label focuses the handle the checkbox tracked. The
   third API, `form::name_here`, is **not** adopted by `Checkbox`, and this is
   the one place I fell short of what was asked. An ambient name has nowhere to
   go in a control that announces no role — a name without a role is dropped
   before it reaches a node — so adopting it means giving `Checkbox`
   `Role::CheckBox`, which `a11y::role_requires_a_name` makes a required-name
   role, which fails on the label-less `checkbox("cb", true)` in
   `src/elements/control_size_tests.rs:63`. That is the a11y rollout in
   `a11y.rs` §6 (`icon_button` first, then `checkbox`), which the review agreed
   is a separate change. `name_here` is unit-tested for inheritance and shadowing
   instead, and `Field` does publish it. Adding the `debug_selector` to
   `Checkbox` — the only way to see "this was disabled by its group" with no
   `aria_disabled` in gpui — follows the precedent `table.rs` set.
2. **State the deferred-draw hazard narrowly.** Done. The module docs now say
   the rule as "read the ambient value in `render` and pass what you read into
   anything you draw out of line; never call `disabled_here` from inside a
   draw-deferring closure", note that `render` runs during `request_layout` and
   is therefore *inside* the scope, and say explicitly that hand-threading
   `disabled(true)` into every popup is not the fix. The case that genuinely
   cannot be handled that way is named: a closure built once and drawn against a
   different ambient value later — a cached panel, or content produced by a
   stored callback the container does not call during layout — has to be given
   the value as an argument, because it and the group are never on the stack
   together. (The prose avoids writing gpui's defer-draw call verbatim:
   `overlay_coverage::every_overlay_is_written_down` matches source text
   including doc comments.)
3. **"A bounded leak" is an assumption about callers.** Fixed. The registry
   comment now says the bound is the number of *distinct field ids ever rendered
   on this thread* — fixed and small for ids written down in source, unbounded
   for a caller deriving an id from a row, a record or a task — and records that
   reference-count eviction was considered and rejected: `FocusHandle` exposes no
   reference count, so "nobody else holds this" cannot be asked from there, and
   dropping a handle a control still tracks would silently unfocus it. The
   available fix is an explicit `clear_field_focus_handles()`, not written until
   something needs it.
4. **Compile the showcase.** Done: `cargo check --example showcase --features
   examples` is clean. It caught nothing, which is the result you want and not a
   reason to have skipped it.

On the two notes about the tree: the base here is trunk at `dc0dd7b` — PR #201
is not in it, so there was no `a11y.rs` conflict to resolve; only the `"field"`
row was deleted and the list is otherwise untouched, unreordered and
unreformatted. `docs/component-triage.md` still lists `Form | Issue |
docs/issues/form.md`, unchanged: flipping it to Shipped moves the row, `EXPECTED`
in `src/elements.rs` and both prose restatements in one commit, and the spec
left that undone on purpose. Nothing fails today.

## Directions from the orchestrator

- **Resist form state.** Nothing pulled toward it, so there is no finding to
  report. The temptation the scope note anticipated did not arise, because the
  two questions this component answers — which group am I in, what am I called —
  are answered by an ambient read at render time with nothing stored anywhere.
- **Read what the a11y half needs before designing the API.** Done, and it
  changed the design: `A11y::active_descendant` is honoured only while a focused
  *ancestor* of the item is on the node stack, "focus stays here and points at a
  sibling" cannot be expressed, and it is the one state field excluded from
  `every_state_field_reaches_the_node`. Label association is exactly where that
  temptation lives, and this change does not go near it: the `Field` announces
  the name itself and republishes it, and the label click moves *real* focus to
  a real handle rather than pointing at a sibling.
- **Check what `field.rs` and `label.rs` already do before adding a third way.**
  Done. `Label::for_id` already exists, is stored and is read by nothing — an
  inert second way. It is left alone rather than deleted (a `Label` is used
  standalone, not only inside a `Field`, and removing a public builder is a
  breaking change this component does not need), but it is worth knowing it is
  there: the association this change ships goes through `Field`, not through it.
- **The test gate at `.tasks/verify`.** `cargo test --lib` is green at 530.
- **Integrate with other builds' branches.** The base turned out to be trunk
  alone — no Calendar, Command or Combobox in it — so there was nothing to
  integrate with. The triage per-status counts were re-read in this tree
  (12 Shipped / 6 Issue / 11 Rejected) rather than copied, and are unchanged
  because Form stays an Issue row.
