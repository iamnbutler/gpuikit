You are a Builder in the Double Diamond architecture.

You are implementing 1 approved spec(s). Verify a spec's claims against the code in front of you; where a spec has a Scout behind it, trust its pitfalls.

## Spec 1 of 1: Form: grouping and label association, not form state (#164)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Form — grouping and label association, not form state

### Summary
Adds `src/elements/form.rs`: a `Fieldset` with a legend, a group-level error, and a
`disabled` that cascades to everything inside it through an **ambient** `FormContext`
rather than a prop threaded by hand; and adopts `Field` onto that context so a field
names the control beside it, publishes the focus handle its label clicks, and inherits
an enclosing fieldset's disabled. `Field` now takes a required id and announces a named
`Role::Group`, which removes it from `a11y::ELEMENTS_WITHOUT_A_ROLE`. No submission, no
validation, no dirty tracking — the line Headless UI draws, and the one #164 argued for.
`cargo test --lib`: **511 passed, 0 failed.**

### Implementation Approach
- **`src/elements/form.rs` (new).**
  - `FormContext { disabled, name, focus_handle }` — what a group tells the controls
    inside it. `over(outer)` defines nesting: `disabled` is OR'd (an inner scope cannot
    re-enable), `name`/`focus_handle` are inherited when unset.
  - `WithFormContext` — the one hand-written `gpui::Element` under `src/elements/`. It
    wraps a single child and pushes the context around `request_layout`, `prepaint` and
    `paint`. It reports no id and no role, so it adds nothing to the a11y tree or the
    element-id path.
  - The stack is a `thread_local!` `Vec<FormContext>` with a `Drop` guard, so a
    panicking child cannot leave it one deep. A `gpui::Global` was rejected: reads would
    need `&mut App`, which puts a `cx` argument on `disabled_here` and takes the ambient
    value straight back to being threaded by hand.
  - Read API: `current()`, `disabled_here(own) -> bool`, `name_here()`,
    `focus_handle_here()`. A control's entire adoption cost is one line in `render`:
    `let disabled = form::disabled_here(self.disabled);`.
  - `Fieldset` / `fieldset(id)`: `legend`, `description`, `error` (the group-level
    summary that had nowhere to go), `Disableable`, `ControlSized`, `ParentElement`.
    Announces `A11y::new(Role::Group).name(legend)`. Children are wrapped in
    `WithFormContext`; legend, description and error sit outside it.
  - `field_focus_handle(&ElementId, &mut App)` — a `thread_local` registry of one
    `FocusHandle` per field id.
- **`src/elements/field.rs`.** `field(id)` / `Field::new(id)` now require an id (breaking;
  `Default` dropped). `impl Accessible` → named `Role::Group`. `render` reads
  `form::disabled_here(self.disabled)`, gives the label its own scoped id
  (`element_id::scoped(&self.id, "label")`), a `cursor_pointer` and an `on_click` that
  calls `window.focus(&handle, cx)`, and wraps the child in a `WithFormContext` carrying
  `disabled` + the label as `name` + that handle.
- **`src/a11y.rs`.** Deleted the `"field"` row from `ELEMENTS_WITHOUT_A_ROLE` — required,
  because `every_element_module_declares_a_role` fails in *both* directions.
- **`src/elements.rs`.** `pub mod form;`.
- **`examples/showcase.rs`.** `("form", "form")` in `ELEMENT_COVERAGE`, nav entry
  `("form", "Form")`, `"form" =>` arm, `render_form_page` (a labelled fieldset with a
  group error; a second fieldset disabled at the group where neither field says
  `disabled`), and the six existing `field()` calls given ids.

### Discovered Pitfalls
- **gpui has no `labelled_by` builder.** grep across gpui 1.16.1 for
  labelled/labeled returns one unrelated doc comment; accesskit has the relation,
  `AriaProperties` has no field. So association is expressed as *the result* of the
  relation — the `Field` publishes its label as the ambient accessible name and the
  control announces it. This is exactly the reason `ELEMENTS_WITHOUT_A_ROLE` recorded
  for `field`, now discharged by a different route than that note assumed.
- **Why an ambient value works at all:** `ViewElement::request_layout` calls
  `RenderOnce::render` and then lays out the result, and `Div` does the same per child —
  so a whole descendant subtree renders *inside* an ancestor's `request_layout`. The
  scope must also be opened in `prepaint`/`paint`, which walk the subtree again.
- **A deferred draw escapes the scope.** `Window::defer_draw` restores the element-id
  stack, not this one. A popover deferred out of a disabled `Fieldset` reads no ambient
  disabled and needs `disabled(true)` by hand. Documented in the module docs.
- **`Window::use_keyed_state` is not usable from `RenderOnce::render`.** It goes through
  `with_element_state`, which `debug_assert!`s paint-or-prepaint; `render` runs during
  `request_layout`. That is why the field focus handle is a registry keyed on the
  (stable, per-`element_id`-rule) field id. It never evicts — a bounded leak, named in
  the code rather than hidden.
- **`overlay_coverage::every_overlay_is_written_down` matches source text, including
  doc comments.** Writing `deferred()` in prose in `form.rs` failed the build until the
  sentence was rephrased. Real, and easy to hit again.
- **No `Cargo.lock` in the repo**, so `^1.14.2` resolves to gpui-unofficial **1.16.1**.
  `#[derive(IntoElement)]` expands to `ViewElement<Self>` there; `a11y.rs`'s docs still
  say `Component<C>` (1.14's name). Docs only, nothing broken.
- **gpui still has no `aria_disabled`.** A disabled control is distinguishable only by
  the `Click` action its node does not offer. Cascading disabled is therefore a *visual
  and behavioural* cascade, not an announced one — unchanged by this work.

### Blockers & Dependencies
- `docs/issues/element-roles-convention.md` — **cleared.** `src/a11y.rs` already is that
  decision (`A11y` + `Announce` + `Accessible`), so this component used it rather than
  inventing a mechanism.
- Upstream gpui: `aria_labelled_by` and `aria_disabled`. Both would be local changes to
  `A11y`/`Announce` when they land.
- Not blocking, but left undone on purpose: `docs/component-triage.md` still lists
  `Form | Issue | docs/issues/form.md`. Flipping it to Shipped means editing the row,
  `EXPECTED` in `src/elements.rs` (12/6/11 → 13/5/11), the two prose restatements of
  those counts, and adding a prose reference to `docs/issues/form.md` so
  `every_written_issue_is_reachable_from_the_triage` keeps passing. Nothing fails today.

### Complexity
Medium

### Notes
- **The remaining half of "cascading disabled" is adoption.** The mechanism is complete
  and tested, and `Fieldset` and `Field` use it, but no leaf control reads it yet:
  `Checkbox`, `Switch`, `Toggle`, `Button`, `TextField`, `Textarea`, `Select`, `Slider`
  each still need the one line `let disabled = form::disabled_here(self.disabled);` in
  `render`. Same for `form::name_here()` (announce it when the control has no name of its
  own) and `form::focus_handle_here()` (use `A11y::focus_handle(h)` instead of
  `focusable()`, which is what makes label-click-to-focus actually land). Until a control
  adopts the handle, a label click focuses a handle nothing tracks — harmless, and inert.
  That adoption is a per-control change and interacts with the a11y rollout order in
  `a11y.rs` §6, which is why it is not bundled here.
- **`field()` is a breaking change** (`field(id)`). Every call site in the repo is
  updated; downstream consumers are not.
- **Verification.** `cargo check --lib` clean apart from a pre-existing warning in
  `src/input/bindings.rs:461`. `cargo test --lib` → 511 passed / 0 failed, including the
  seven new unit tests in `form.rs` (empty stack, scope visibility, nesting cannot
  re-enable, name inheritance, guard-through-panic, fieldset announcement, derived
  legend id) and three in `field.rs`, plus every pre-existing coverage test
  (`showcase_coverage`, `every_element_module_declares_a_role`,
  `no_element_calls_gpuis_a11y_builders_directly`, `no_element_mints_a_constant_id`,
  `overlay_coverage`, `triage_coverage`).
- **The showcase example was not compiled.** It is behind `--features examples` and a
  full link of gpui did not fit the run's budget. `showcase_coverage` proves the table,
  the nav entry and the match arm are consistent; it does not prove `render_form_page`
  type-checks. That is the one thing a Builder should compile first:
  `cargo check --example showcase --features examples`.

## Review feedback on these specs

A reviewer read the spec(s) above and approved them **with** the following. It is part of what was approved: the spec says what to build, this says what the reviewer required of it. It is not part of any spec text, so nothing above repeats it.

Treat every item as a requirement, not a suggestion. Where one genuinely conflicts with the spec it was written about, the feedback wins — it is the later word, written by the person who approved that spec — but **say so in `SUMMARY.md`**.

Account for every item in `SUMMARY.md` under a `## Review feedback` heading: one line per item saying you did it, or that you decided against it and why. Declines are fine and are expected to be written down; an item you silently dropped is indistinguishable from one you never read, and the reviewer reads the spec rather than this section.

### On spec 1 of 1: Form: grouping and label association, not form state (#164)

Approved. `511 passed, 0 failed` with seven new unit tests including the guard-through-panic one, and the `Global` alternative rejected for a stated reason rather than by default — this is careful work. Four items, each accounted for in SUMMARY.md.

1. NOTHING THE ISSUE ASKED FOR WORKS UNTIL ONE CONTROL ADOPTS THE CONTEXT — ADOPT ONE HERE. Your own note is the finding: no leaf control reads `disabled_here`, `name_here` or `focus_handle_here`, so after this lands a fieldset's `disabled` cascades to nothing, a label click focuses a handle nothing tracks, and the label association in the issue's title does not exist. What ships is a mechanism with no consumer, which is the state that looks finished and is not.

   I am not asking for all eight — that is a separate change and you are right that it interacts with the rollout order in `a11y.rs` §6. Adopt **one**, whichever is cheapest (`Checkbox` is the obvious candidate: it has a `Disableable`, a name, and a focus handle), and write the end-to-end test that a mechanism deserves — a disabled `Fieldset` containing a `Checkbox` that says nothing about `disabled` renders it disabled, and a click on its `Field` label lands focus on it. One working path proves the three read APIs at once; seven unit tests over the stack prove the stack.

2. YOUR DEFERRED-DRAW HAZARD IS REAL AND NARROWER THAN YOU STATED — SAY WHICH. "A popover deferred out of a disabled `Fieldset` reads no ambient disabled" is true of a read performed *inside* the deferred closure. It is not true of a read in `render`, which runs during `request_layout`, inside the scope — and `render` is exactly where your adoption instruction puts it. So the rule is not "deferred elements are broken", it is **read the ambient value in `render` and pass it into whatever you defer; never call `disabled_here()` inside a deferred closure.** Write it that way in the module docs. The version in the spec reads as an unavoidable hole and would teach the next author to hand-thread `disabled(true)` into every popup, which is the threading the whole design exists to remove. If there is a case where the deferred closure genuinely must read it, name that case.

3. "A BOUNDED LEAK" IS AN ASSUMPTION ABOUT CALLERS, NOT A PROPERTY. The focus-handle registry never evicts, so it is bounded by the number of distinct field ids *ever rendered*, not by the number of fields on screen. That is fine for a form with fixed ids and unbounded for any caller that derives a field id from a row, a record or a task — which is the ordinary way an id gets made in a list-driven app. Keep the registry; change the comment to say what the bound actually is and what makes it grow, so nobody reads "bounded" and stops thinking. If there is a cheap eviction (dropping an entry whose only remaining `FocusHandle` reference is the registry's), say whether you considered it and why not.

4. COMPILE THE SHOWCASE. `cargo check --example showcase --features examples` — `showcase_coverage` proves the table, the nav entry and the match arm agree with each other, and proves nothing about whether `render_form_page` type-checks. You identified this as the first thing to do; do it. #157's scout ran `cargo test --lib` to 512 *and* this command clean in the same run, so it fits.

Two things about the tree you will build on. `src/a11y.rs` is also edited by PR #201, which is open and unmerged — #165 deletes the `"dialog"` row from `ELEMENTS_WITHOUT_A_ROLE` and you delete the `"field"` row from the same list. Expect to resolve that, and do not take the opportunity to reorder or reformat the list. And `docs/component-triage.md`: you are right that nothing fails today, but if you flip Form to Shipped, the row, `EXPECTED` in `src/elements.rs`, and both prose restatements move in one commit or the build fails three ways at once.

## Directions for this implementation

The orchestrator agent added the following when requesting this build. It is **not** part of any spec above, and no reviewer has seen it — it is addressed to you.

Treat it as a requirement, not a suggestion. The specs are still what is being implemented; these directions say how to go about it. Where one genuinely conflicts with a spec, the direction wins — it was written after the spec was approved, with this build in view — but **say so in `SUMMARY.md`**, because the reviewer reads the spec and cannot see this section.

Account for every direction in `SUMMARY.md` — including any you decided against, and why. A direction you silently dropped is indistinguishable from one you never read.

This is the odd one out of the four component specs and the title says why: **grouping and label association, not form state**. Resist the pull toward validation, dirty tracking, submission or a form-level value store — every one of those is a decision about someone else's architecture, and a toolkit that makes it is one people fight rather than use. If you find yourself wanting a `FormState`, stop and write in `SUMMARY.md` what pushed you there; that is a finding worth having and not a feature worth shipping quietly.

What the accessibility half actually needs is worth reading before you design the API. `src/a11y.rs` documents that `active_descendant` is honoured only while a focused **ancestor** of the item is on the node stack, and that the arrangement where focus stays on one element and points at a sibling **cannot be expressed** — it is dropped in silence, and it is the one state field excluded from `every_state_field_reaches_the_node`, so nothing can read it back to catch the mistake. Label association is exactly the place that temptation arises. Check what `src/elements/field.rs` and `src/elements/label.rs` already do before adding a third way.

**This repository now has a test gate.** `.tasks/verify` is on the trunk as of `dc0dd7b9` and is `exec cargo test --lib`, so a red suite fails your build inside the VM and opens no pull request. It ran ten consecutive times green at 517 tests on the merge, so a failure is yours rather than inherited. `cargo clippy` on the trunk is 30 warnings and zero errors, all pre-existing; the gate does not run clippy and cleaning those up is not yours.

Your base is likely to be other builds' branches carrying Calendar (#157), Command (#159) and Combobox (#160), all of which edit `examples/showcase.rs` and `src/elements.rs`. Integrate with what you find rather than beside it, and re-read `docs/component-triage.md`'s per-status counts in the tree you are given — they move with every component that changes status and a number copied from elsewhere will be stale.

## Your job

1. Implement every spec above, in order, as one coherent change in the cloned repo (cwd). You are on the right branch already.
2. Run the project's tests / lint / typecheck — get them green.
3. Commit your work with clear messages (a git identity is configured).
4. Write `SUMMARY.md` in the repo root: one or two paragraphs describing the change, suitable as a pull request body. Do not use GitHub closing keywords (`Closes #N`, `Fixes #N`) — the server links the issues itself.
5. Do NOT push and do NOT open a PR — the server does both.

**You have 60 minutes, once.** That is the whole run — the clone before you started, this turn, the supervisor's own test run and the packaging after it — measured on the wall clock from dispatch. There is no later: when you end your turn the run is over. A backgrounded command buys you nothing — its child is killed with the turn — so anything whose result you need must be awaited inline, and a poll loop over a file another process will write can only report to a turn that has already ended. Nor should you start what cannot finish: a cold build in a large workspace can run forty minutes, so weigh what a command will cost against what is left.

On step 2: when this project declares a test suite at `.tasks/verify`, the supervisor runs it itself after you finish, against the committed tree your branch carries. If it fails you get one chance to fix it and then the build fails with no pull request, so getting there first is entirely in your interest. It reads that script out of the build's BASE commit, so editing it changes nothing about what runs.
