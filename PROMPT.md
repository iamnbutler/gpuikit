You are a Builder in the Double Diamond architecture.

You are implementing 3 approved spec(s). Verify a spec's claims against the code in front of you; where a spec has a Scout behind it, trust its pitfalls.

## Spec 1 of 3: Adopt Splitter into the focus convention instead of its own tab_index (#181)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Adopt Splitter into the a11y focus convention instead of its own `tab_index`

### Summary
`Splitter` reached the tab order through a raw `tab_index(0)` on its band — a
mechanism nothing else in the crate uses — while every other keyboard-operable
control declares focus on its `A11y` and lets `Announce::announce` apply it.
Both #173 and #158 are now in the tree, so the deferral can be closed:
`Role::Splitter` joins `a11y::role_requires_keyboard_focus`, `Splitter`'s
announcement declares `.focus_handle(handle)` (the element already owns a
handle, because a mouse-down on the band has to focus it), and the band's
`tab_index(0)` — and the `track_focus` that `announce` now performs itself —
are deleted. The arrow / `home` / `end` contract is untouched; only the route
into the tab order changes.

### Implementation Approach
- **`src/a11y.rs`**
  - Add `| Role::Splitter` to `role_requires_keyboard_focus`, between `Slider`
    and `SpinButton`, with a comment saying why: a standalone control that owns
    one tab stop and moves a value with the arrow keys — `Slider`'s shape.
  - Delete the `Role::Splitter` decline bullet from that function's docs (the
    remaining two groups — composite items, landmarks — become "Two groups").
  - Flip `the_focus_rule_covers_the_roles_a_keyboard_operates`: the
    `assert!(!role_requires_keyboard_focus(Role::Splitter))` becomes a positive
    assertion carrying the argument for the arm.
- **`src/elements/splitter.rs`**
  - `Splitter::announcement` takes a third argument, `focus: Option<FocusHandle>`,
    and ends in `match focus { Some(h) => a11y.focus_handle(h), None => a11y.focusable() }`.
    It is the one announcement builder, shared by `Accessible::a11y` (no window,
    no handle → `focusable()`) and `render` (the band's handle → `focus_handle(h)`).
    Both are `Focus::Takes`, so `is_focusable()` is true and
    `is_missing_a_focus_decision()` false on either path.
  - `render` passes `Some(focus_handle.clone())` — the handle `SplitterState`
    already mints and that `on_mouse_down` already focuses.
  - The band drops `.tab_index(0)` **and** `.track_focus(&focus_handle)`, with a
    comment saying `announce` now does both. Everything else on the band —
    hover, cursor, the `Separator` child, mouse-down, `on_key_down` — is
    unchanged.
  - Module docs gain a paragraph under `# Accessibility` stating that the band
    is a tab stop by declaration, and why the element hands over its own handle
    rather than letting gpui mint one.
  - Two tests: `a_splitter_declares_that_it_takes_keyboard_focus` (the
    declaration, no window) and `tab_reaches_the_band` (the tab order, drawn).
- **`CHANGELOG.md`** — both entries are still under `## [Unreleased]`, so the
  #173 bullet's Splitter decline is replaced rather than appended to, and the
  `Role::Splitter` bullet in the splitter entry now records the adoption.

### Discovered Pitfalls
- **`announced()` cannot see any of this.** It calls two `Element` methods and
  never lays out or paints, so neither the tracked handle nor the tab-stop
  registration exists for it to look at. `tab_reaches_the_band` therefore draws
  a real window and presses a real Tab.
- **The band's own `track_focus` had to go, not just the `tab_index`.**
  `announce` applies `track_focus(&handle.tab_stop(true))`; a second
  `track_focus(&focus_handle)` after it would put the plain, non-stop handle
  back on the element and silently take the splitter out of the tab order
  again. `track_focus` does not carry the element's `tab_stop` onto the handle
  — only the minted path does.
- **`focusable()` was not an option for the render path.** A mouse-down on the
  band calls `focus_handle.focus(window, cx)`, and a handle gpui mints into
  element state is not reachable from a listener. Hence `focus_handle(h)`, and
  hence `announcement` having to admit both a handle and its absence.
- **The existing splitter test harness cannot press Tab.** `draw` calls
  `crate::theme::init`, and Tab is bound by `crate::init` →
  `a11y::bind_focus_keys`; gpui binds `focus_next`/`focus_prev` to nothing. The
  new test calls `crate::init` first and wraps the build in a root that
  `track_focus`es a handle it then focuses — with nothing focused, gpui
  dispatches `FocusNext` to the node above the root element and the first Tab
  reaches no listener. This is `elements::button`'s harness shape.
- **"Focus moved" is weaker than "the band has focus."** After Tab the test
  presses `right` and asserts the ratio moved past 0.5, because the arrow keys
  are the band's own `on_key_down` listener — a divider that moves is a divider
  the keyboard actually reached.
- `announce` now also puts `moves_focus_on_tab()` on the band, which the
  `tab_index` path did not: Tab out of a focused splitter is answered by the
  band itself rather than relying on an ancestor listener. Strictly an
  improvement, but it is a behavioural difference worth knowing about.

### Blockers & Dependencies
None remain. #173 (`a11y`'s focus convention) and #158 (`Splitter`) are both in
the tree on this branch — verified by reading `role_requires_keyboard_focus`,
`Announce::announce` and `src/elements/splitter.rs` — so this issue was only
ever ordering. Nothing here depends on the roving-focus convention the
composite-item roles are waiting for; a splitter owns exactly one tab stop.

### Complexity
Simple

### Notes
- **What was verified, and what was not.** The clone has no `target/` and no
  cargo registry, so `cargo check`/`cargo test` would have had to fetch and
  compile gpui + gpui_platform from cold on a 4-core, 5 GB box, with `x11` and
  `wayland-client` absent from `pkg-config`. That does not fit in this run's
  budget and would have crowded out the spec, so it was not attempted. What was
  run: `cargo fmt -- --check`, clean — which also confirms all three files
  parse. Everything else is reasoned against the code in the tree, and every
  API the new test uses is used the same way in `elements::button`'s keyboard
  tests (`crate::init`, `track_focus` + `moves_focus_on_tab` on the root,
  `window.focus`, `window.focused`, `simulate_keystrokes`).
- **First thing the Builder should run**: `cargo test --lib splitter` and
  `cargo test --lib a11y`. The three tests that pin this change are
  `a11y::tests::the_focus_rule_covers_the_roles_a_keyboard_operates`,
  `elements::splitter::tests::a_splitter_declares_that_it_takes_keyboard_focus`
  and `elements::splitter::tests::tab_reaches_the_band`. The existing splitter
  announcement tests were touched only by the new `announcement` argument
  (`None`), and the drag tests not at all.
- If `tab_reaches_the_band` proves flaky about geometry, the cause is the
  canvas measuring during paint: the arrow keys do nothing until
  `container_bounds` is `Some`, which is the frame after the first. The test
  parks after `draw`, after focusing the root, and after Tab, which is more
  parking than `scenario` needs — but that is the knob.
- `Sidebar` is still not migrated onto `Splitter`; unrelated, and its own
  decision (`docs/component-triage.md`).

## Spec 2 of 3: role_requires_keyboard_focus pins 4 of its 16 arms, so 12 roles can silently fall off the focus rule (#195)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Pin all 16 arms of `role_requires_keyboard_focus` with one assertion per reason

### Summary
`role_requires_keyboard_focus` in `src/a11y.rs` has 16 arms, and
`tests::the_focus_rule_covers_the_roles_a_keyboard_operates` pinned only 4 of
them (`Button`, `DefaultButton`, `ComboBox`, `TextInput`) — the test's other six
named roles were *non*-membership assertions, so the ten-role list read like
coverage while holding a quarter of the rule in place. The remaining 12 arms
could be deleted with the suite still green, and quietly, because the rule feeds
a `debug_assert!` that no other test observes and that a release build drops
entirely. This change rewrites that test to assert membership for all 16 arms,
grouped by reason with the reason interpolated into the failure message, copying
the shape #185 already landed for `role_requires_a_name`'s 28 arms; it also adds
the matching "the rot is quiet / every arm is pinned" paragraph to the rule's own
doc comment. The non-membership half deliberately stays a sample, for the reason
#185 gives, but is widened from the six roles the old test happened to name to
the ten the rule's own docs argue for in writing.

### Implementation Approach
- **`src/a11y.rs` is the only file touched.** Test code and doc comments only —
  no behaviour change, the `matches!` arms are untouched.
- **`the_focus_rule_covers_the_roles_a_keyboard_operates` rewritten** with
  membership exhaustive over all 16 arms, one `for role in [...] { assert!(...) }`
  per reason group and the reason as the message with `{role:?}` interpolated:
  - `Button`, `Link` — activated by a keystroke on the control itself, and a
    keystroke only reaches the focused element.
  - `DefaultButton` — a standalone `assert!`, keeping verbatim the "a dialog's
    Enter key resolves to the default button, so it is the most focus-requiring
    control in the set" wording that was already both in the test and in the
    rule's inline comment. (This mirrors how #185 gives `Role::Splitter` its own
    assertion rather than forcing it into a group.)
  - `CheckBox`, `Switch`, `RadioButton` — a state the keyboard changes with
    Space, so the toggle is a keystroke that has to land on the control.
  - `Slider`, `SpinButton` — value moved with the arrow keys, which go to the
    focused element and nowhere else.
  - `ComboBox`, `EditableComboBox` — the popup's arrow keys are delivered to the
    trigger, so the trigger holding focus is what makes the list operable.
  - `TextInput`, `MultilineTextInput`, `SearchInput`, `NumberInput`,
    `PasswordInput`, `DateInput` — typed into, and typing is nothing but the
    focused element's keystrokes.
  2 + 1 + 3 + 2 + 2 + 6 = 16.
- **The test's doc comment** states the membership/non-membership asymmetry
  explicitly, the way #185's does, and records that the test used to name four —
  so the next reader knows the exhaustiveness is load-bearing rather than
  incidental.
- **Non-membership kept a sample, widened to the argued set.** #185's argument
  applies unchanged: exhaustive non-membership would mean asserting against all
  182 of accesskit 0.24.1's `Role` variants, almost none of which this crate has
  said anything about, and an assertion with no argument behind it is the failure
  mode the module is organised against. So the negative half asserts *arguments*,
  not roles — which is why it now covers exactly the three groups the rule's own
  docs argue for: all six composite-item roles (adding `MenuItemCheckBox` and
  `MenuItemRadio`), `Splitter` (with the gpuikit#181 reason spelled out), and all
  three landmarks/containers (adding `Document` and `Group`). Previously it named
  an arbitrary four-plus-one-plus-one subset of those ten.
- **`role_requires_keyboard_focus`'s doc comment** gains a closing paragraph
  matching `role_requires_a_name`'s: the rot is quiet, every arm is pinned by
  name in the test, and adding an arm means adding it there too with the argument
  as the message.

### Discovered Pitfalls
- **#185 has already landed** — commit `23f2d40`, and the exhaustive
  `the_name_rule_covers_the_roles_that_are_nothing_without_one` is in the tree.
  The issue's "wait for #185 to land and copy its shape" is therefore satisfied;
  there is no blocking dependency left, and the shape to copy is at
  `src/a11y.rs:1323` onwards.
- The two rules' arm lists are *not* the same set, so the name test's groups
  cannot be copied wholesale. `role_requires_a_name` has 28 arms and includes the
  composite-item roles, `Splitter`, `ProgressIndicator`, `Meter`, `Dialog`,
  `AlertDialog` and `Image`; the focus rule has 16 and excludes all of those. Only
  the *form* transfers — the reasons had to be written for the keyboard, not
  reused from the name rule.
- `Role::Group` and `Role::Document` were not used as real variants anywhere in
  `src/a11y.rs` before this change (only inside string literals in the
  role-adoption ledger), so their existence was checked directly against
  accesskit 0.24.1's `Role` enum in the fetched registry source rather than
  assumed. Both exist, as do all ten roles on the negative side.
- The rule's arms carry an inline comment for `DefaultButton`. Its wording is now
  duplicated in the test's standalone assertion; that duplication is intentional
  and matches how `Splitter`'s argument appears in both places for the name rule.
- The repo has no `Cargo.lock` and, on a fresh machine, no cargo registry, so the
  first build downloads and compiles all of gpui.

### Blockers & Dependencies
None outstanding. #185 (the `role_requires_a_name` pinning this copies) is
already merged as `23f2d40`. iamnbutler/gpuikit#181 (`Splitter` adopting the
focus convention) is *referenced* by the negative assertion but does not block
this change — when #181 lands, `Splitter` moves from the negative block to a
membership assertion, and the assertion message here says why it is not one yet.

### Complexity
Simple

### Notes
- Verified, not just written: cold build 7m11s on 4 cores, then
  - `cargo test --lib the_focus_rule_covers` → ok;
  - **mutation check** — deleting `Role::Slider` from the rule makes the test
    fail with `Slider moves its value with the arrow keys, ...`, i.e. the reason
    is the failure message; the arm was restored and the file diffed
    byte-identical to its pre-mutation state;
  - `cargo test --lib a11y::` → 19 passed;
  - `cargo test --lib` → 501 passed, 0 failed;
  - `rustfmt --edition 2021 --check src/a11y.rs` → no diff.
- The one pre-existing warning in the build (`unused_mut` at
  `src/input/bindings.rs:461`) is untouched by this change and was there before.
- If a future arm is added to the rule, the test will not fail — nothing forces
  a *new* arm to be pinned, only a *dropped* one to be caught. That is the same
  limit `role_requires_a_name`'s test has, and the same answer applies: what
  catches a wrong addition is the review that adds it. Making it airtight would
  need the rule to be a `const` slice the test iterates, which is a larger design
  change than either issue asked for and would lose the `matches!` form both
  rules share.

## Spec 3 of 3: Confirmation dialog: the destructive-action affordance Dialog is missing (#165)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Confirmation (alert) dialog as a `Dialog` mode

### Summary

`src/elements/dialog.rs` already shipped the modal half of #59's "Alert
Dialog" — a scrim, a centred panel, a title, a close button, Escape and
backdrop dismissal. What was missing is the *confirmation* affordance: a
question, two answers of which one destroys something, focus that lands on
the safe answer, and an announcement that reads as one alert rather than as a
window containing some text. This adds that as a **mode of the existing
`Dialog`** rather than a second modal implementation, and adds the destructive
`ButtonVariant` it depends on first. Everything compiles (`cargo check --lib`,
`--tests`, and `--example showcase --features examples` are all clean); the
test *binary* could not be linked inside this run's budget, so the eight new
tests are type-checked but unrun — see **Notes**.

### Implementation Approach

**1. `ButtonVariant::Destructive` (the prerequisite, and the first commit).**

- `src/elements/button.rs`: `ButtonVariant` grows a `Destructive` arm and
  derives `Debug, Clone, Copy, PartialEq, Eq`; `Button` grows a `variant` field,
  a `variant(…)` builder and a `.destructive()` shorthand. The `// todo: style
  through ButtonVariant` is discharged: background, text colour, hover, active
  and the focus ring all branch on the variant. `traits::button::Button::variant`
  now returns the stored value instead of `Default::default()`.
- The inherent `variant(…)` builder **shadows** the trait getter of the same
  name for method-call syntax. Nothing in the crate called the getter (grepped),
  and the doc comment plus a test say how to reach it
  (`traits::button::Button::variant(&b)`).
- `src/theme/mod.rs`: `destructive_bg`, `destructive_bg_hover`,
  `destructive_bg_active`, `destructive_fg` as `Themeable` defaults in the
  "Component-specific defaults" section, derived from the existing `danger()`
  rather than introduced as new palette entries. `destructive_fg` picks black or
  white off the background's lightness rather than using `fg()`, which in a
  light theme would sit dark-on-red.

**2. Confirmation as a `Dialog` mode.**

- `Confirmation { confirm_label, cancel_label, destructive, on_confirm,
  on_cancel }`, held as `Dialog::confirmation: Option<Confirmation>` — one
  `Option` rather than five fields that could disagree about whether this is a
  confirmation.
- `dialog(id).confirm(question, answer)` sets title, description and mode in
  **one call**, which is how "title and description as a pair" is enforced: the
  two are independent `Option`s everywhere else in the builder, and a
  confirmation cannot be built with one and not the other. It also turns the
  header's X off, since Cancel is already on screen.
- `.confirm_label` / `.cancel_label` / `.destructive` / `.on_confirm` /
  `.on_cancel` refine the confirmation and **refuse to create one**
  (`debug_assert!` via a private `with_confirmation`), so
  `.confirm_label("Delete")` alone cannot produce an `AlertDialog` with no
  question in it.
- `DialogState` gains `confirm()`, `cancel()`, a private `dismiss()`, the
  `DialogConfirmed` / `DialogCancelled` events (alongside the existing
  `DialogOpened` / `DialogClosed` / `DismissEvent`), and `is_confirmation()`.
  Both `confirm` and `cancel` close first, then emit, then run the handler.
- **Escape, the backdrop and the header's close button all route through one
  function**, `dismiss`, which cancels a confirmation and plainly closes a
  plain dialog. There is no path in the file from a key or a stray click to
  `DialogConfirmed`; only the confirm button reaches it.
- **Initial focus on the safe action.** `DialogState::open` mints a second
  `FocusHandle` for the cancel button and focuses it, and `render` hands it to
  that `Button` through the existing `Button::focus_handle` — the API whose doc
  comment says it exists for exactly this. The root handle is still focused
  first and still tracked, because it is what carries `DIALOG_CONTEXT` and the
  `Close` action; the cancel button is a descendant of the element tracking it,
  so Escape still dispatches to this view after focus moves down.
- The confirmation renders its own footer (Cancel then Confirm, safe answer
  first). A caller-supplied `.footer()` is ignored in confirm mode rather than
  stacked underneath.
- `impl Accessible for DialogState` → `Role::AlertDialog` when confirming,
  `Role::Dialog` otherwise, named by the title and described by the
  description. `announce` goes on the **panel**, not the scrim, because the
  panel is what the title names.
- `impl ControlSized for Dialog`: the rung sizes the two action buttons and the
  footer gap. See the pitfall below on why the panel itself is not on the scale.
- `src/a11y.rs`: the `"dialog"` entry is deleted from
  `ELEMENTS_WITHOUT_A_ROLE`. `src/elements/control_size_tests.rs`'s prose count
  of `ControlSized` implementors goes sixteen → seventeen, with `Dialog` added
  to its list of deliberately-unmeasured ones.
- `examples/showcase.rs`: a second, destructive confirmation on the existing
  Dialog page (`destructive_dialog`), opened by a `.destructive()` button.

### Discovered Pitfalls

- **The issue body's accessibility section is stale.**
  `docs/issues/element-roles-convention.md` is marked *Settled*; `src/a11y.rs`
  (1808 lines) is the decision, and `Button`, `Sidebar`, `Select` and
  `Splitter` are already on it. Do not invent a mechanism — implement
  `traits::accessible::Accessible` and call `.announce(a11y)`.
- **`role_requires_a_name` covers `Role::Dialog` *and* `Role::AlertDialog`.**
  An untitled dialog would therefore trip a `debug_assert!` inside `announce`.
  `render` guards the announcement on `title.is_some()`, so an untitled plain
  dialog announces **nothing** rather than an unnamed role. A confirmation
  cannot reach that path — its title is a constructor argument. A test asserts
  the guard is not dead code.
- **`a11y::tests::every_element_module_declares_a_role` is a textual scan** for
  a line containing both `impl` and `Accessible for`. Adopting the module
  *forces* its `ELEMENTS_WITHOUT_A_ROLE` entry to be deleted — there is no way
  to announce only in confirm mode and stay excused. That entry's stated reason
  was the missing `aria_modal`; that trade is re-taken (see below) and the
  argument is re-recorded in `dialog.rs` next to the `Accessible` impl, so it is
  not lost.
- **gpui still has no `aria_modal`.** Confirmed by reading
  `gpui-unofficial-1.16.1/src/elements/div.rs`: the `fn aria_*` family is
  `aria_label`, `aria_description`, `aria_keyshortcuts`,
  `aria_active_descendant`, `aria_selected`, `aria_expanded`, `aria_toggled`,
  the numeric-value family, `aria_orientation`, `aria_level`,
  `aria_position_in_set`, `aria_size_of_set`, and the row/column family. No
  modal. And `a11y::tests::no_element_calls_gpuis_a11y_builders_directly`
  forbids reaching around `A11y` to fake one. Modality is therefore carried by
  the scrim and by `Role::AlertDialog` alone; `aria_modal` is an upstream ask
  in the same drawer as `aria_disabled` and `aria_sort`.
- **`track_focus` does not make a caller-supplied handle a tab stop** —
  `announce` does (a11y module docs, section 4). Moving focus to the cancel
  button therefore has to go through `Button::focus_handle`, not a hand-rolled
  `track_focus` on a wrapper.
- **The inherent-vs-trait `variant` collision** is real and bites in test code
  first: `b.variant()` with no arguments resolves to the inherent builder and
  fails with "takes 1 argument but 0 were supplied". Use
  `traits::button::Button::variant(&b)`.
- **A dialog is not a row control.** The issue's boilerplate sizing section says
  "take every dimension from the rung"; taken literally for a modal surface that
  would mean giving a dialog a 20px height. The rung is applied where it means
  something — the two action buttons and the gap between them — and the panel's
  own padding stays component-specific, which is what `src/theme/control.rs`'s
  "What belongs here" note actually asks for.
- **No new `pub mod`**, so `elements::showcase_coverage` needs no new
  `ELEMENT_COVERAGE` row; the confirmation demo goes on the existing Dialog
  page. Adding a row for a page no match arm renders would have failed the
  second of those two tests.
- `cargo check --example showcase` fails with "requires the features:
  `examples`" — it needs `--features examples`.

### Blockers & Dependencies

Both blockers named in the issue are cleared:

- **A destructive `ButtonVariant`** — delivered here, as the first commit.
- **`docs/issues/element-roles-convention.md`** — already settled and shipped
  as `src/a11y.rs` before this issue was picked up.

Nothing else blocks this. Two things it deliberately does *not* do, which are
natural follow-ons rather than parts of it:

- **Focus is not trapped inside the modal.** Tab can still walk out of an open
  dialog into the page behind the scrim. That is a `Dialog`-wide gap that
  predates this change, it needs a focus-trap mechanism the crate does not have,
  and it wants its own issue.
- `Role::Alert` / the live-region decision that `elements::alert` is waiting on
  is untouched; its `ELEMENTS_WITHOUT_A_ROLE` entry still mentions "AlertDialog
  for the modal shape" and is now slightly out of date in wording, though not in
  substance.

### Complexity

Medium

### Notes

- **The tests are type-checked but were not run.** `cargo check --lib`,
  `cargo check --tests` and `cargo check --example showcase --features examples`
  are all clean. `cargo test --lib elements::dialog` requires a full codegen
  link of gpui with `test-support` — the target directory held only `.rmeta` for
  it — which does not fit the remaining budget. Eight tests were added to
  `src/elements/dialog.rs::tests`: the alert-dialog announcement compared as a
  whole `A11y` value (which is how the description is pinned, since `A11y`
  exposes no description getter but does derive `PartialEq`), the plain-dialog
  role, the untitled-dialog guard, the "label builders do not conjure a
  confirmation" rule, the button variant getter, and four behavioural ones —
  Escape cancels and never confirms, the backdrop takes the same route,
  confirming runs its handler exactly once and is a no-op afterwards, and a
  confirmation opens with the cancel handle focused. Run
  `cargo test --lib elements::dialog` first.
- The focus assertion in `a_confirmation_opens_with_the_safe_action_focused` is
  the one most likely to need adjusting: it reads `window.focused` immediately
  after `open` on an entity that is never drawn. If gpui clears focus for a
  handle that is not in the painted tree, that test needs the real-window
  `Harness` pattern from `elements::button::tests` / `elements::sidebar` instead
  — a role plus a mouse listener cannot be drawn with
  `VisualTestContext::draw`, so it needs an actual view.
- `Confirmation`'s `destructive` defaults to `true`. A caller who has not
  thought about it gets the louder answer rather than two identical buttons.
  `.destructive(false)` is there for a confirmation that merely needs
  acknowledging.
- `confirm_label` defaults to "Confirm", but the doc comment argues for naming
  the verb ("Delete", "Discard"), and the showcase does.

## Review feedback on these specs

A reviewer read the spec(s) above and approved them **with** the following. It is part of what was approved: the spec says what to build, this says what the reviewer required of it. It is not part of any spec text, so nothing above repeats it.

Treat every item as a requirement, not a suggestion. Where one genuinely conflicts with the spec it was written about, the feedback wins — it is the later word, written by the person who approved that spec — but **say so in `SUMMARY.md`**.

Account for every item in `SUMMARY.md` under a `## Review feedback` heading: one line per item saying you did it, or that you decided against it and why. Declines are fine and are expected to be written down; an item you silently dropped is indistinguishable from one you never read, and the reviewer reads the spec rather than this section.

### On spec 1 of 3: Adopt Splitter into the focus convention instead of its own tab_index (#181)

Approved. The reasoning is sound and the three pitfalls are the valuable part — particularly that the band's own `track_focus` had to go along with the `tab_index`, because `announce` applies `track_focus(&handle.tab_stop(true))` and a second plain `track_focus` after it would put the non-stop handle back and silently take the splitter out of the tab order again. That is the failure this change would otherwise have shipped looking correct. Three items for SUMMARY.md.

1. THE BUILD FITS — RUN THE TESTS. This spec was not compiled, on the reasoning that a cold gpui build does not fit the budget and that `x11`/`wayland-client` are missing from `pkg-config`. A second scout in this same repository, in the same conditions, minutes later, did it: **cold build 7m11s on 4 cores**, then `cargo test --lib` green at 501 passed and `cargo test --lib a11y::` at 19. So the premise was wrong and there is nothing here you cannot verify. Run `cargo test --lib splitter` and `cargo test --lib a11y`, and report the results — `tab_reaches_the_band` in particular, since it is the one test that draws a real window and presses a real Tab, and it is the only evidence that the tab stop actually exists rather than being declared.

2. THIS SHARES ITS TEST WITH #195, WHICH IS ALSO APPROVED AND WHICH I INTEND TO BATCH WITH IT. spec_2c5232e10a424c14a5d5d47ac1d3f98e rewrites `the_focus_rule_covers_the_roles_a_keyboard_operates` to assert membership for all sixteen arms grouped by reason, and keeps `Splitter` in the widened negative block with your issue number spelled out as the reason. Apply that change FIRST and this one on top: `Role::Splitter` becomes a seventeenth membership assertion carrying the arrow-key argument, and leaves the negative block. Do not write the flip against the old four-role version of the test. If you end up carrying only one of the two, say which in SUMMARY.md and leave the other's half alone.

3. NAME THE BEHAVIOURAL CHANGE IN THE PR BODY, NOT JUST IN THE SPEC. `announce` also applies `moves_focus_on_tab()`, which the `tab_index` path did not, so Tab *out* of a focused splitter is now answered by the band rather than by an ancestor listener. The spec calls it strictly an improvement and I agree, but it is the one thing here that changes behaviour beyond the route into the tab order, and a reviewer should not have to find it in a pitfalls list.

I checked the premise that matters: `role_requires_keyboard_focus` at `src/a11y.rs:669` has sixteen arms today and `Role::Splitter` is not among them, so the adoption is real work rather than a no-op. Keep `announcement` as the one shared builder taking `Option<FocusHandle>` — two builders, one per path, is how the two callers come to disagree about `Focus::Takes`.

### On spec 2 of 3: role_requires_keyboard_focus pins 4 of its 16 arms, so 12 roles can silently fall off the focus rule (#195)

Approved. This is the shape more specs should have: the rule was rewritten, then a **mutation check** confirmed the rewrite actually holds — deleting `Role::Slider` fails the test with the reason as the message, and the file was restored byte-identical. That is the difference between a test that exists and a test that works, and it should be in SUMMARY.md.

I verified the arithmetic rather than trusting it: `role_requires_keyboard_focus` at `src/a11y.rs:669` has exactly the sixteen arms this spec names, in the groups it names — 2 + 1 + 3 + 2 + 2 + 6. Nothing to correct.

Two items for SUMMARY.md.

1. THIS SHARES ITS TEST WITH #181, WHICH IS ALSO APPROVED. spec_9c8bbbf8de37408bb64f3ff35922c306 adopts `Role::Splitter` INTO the focus rule and flips exactly the negative assertion this spec is widening. I intend to batch the two into one build, in which case apply this change first — the exhaustive membership rewrite over sixteen arms — and #181's adoption on top of it, so `Splitter` arrives as a seventeenth membership assertion carrying its arrow-key argument and leaves the negative block. If you find yourself carrying only one of the two, say which, and leave the other's half of the test alone rather than guessing at it.

2. KEEP THE NON-MEMBERSHIP HALF A SAMPLE. Widening it to the ten roles the docs actually argue for is right; making it exhaustive over accesskit's 182 variants would be asserting things nobody has a reason for, which is what this module is organised against. The spec says so and I am recording that I agree, so a later reader does not read the asymmetry as an oversight.

The stated limit is honest and I am not asking for more: nothing forces a *new* arm to be pinned, only a dropped one to be caught. Making that airtight would mean turning the rule into a const slice and losing the `matches!` form both rules share — a larger change than this issue asked for.

### On spec 3 of 3: Confirmation dialog: the destructive-action affordance Dialog is missing (#165)

Approved. Building it as a mode of the existing `Dialog` rather than a second modal is right, and so is routing Escape, the backdrop and the header X through one `dismiss` so there is no path from a stray key to `DialogConfirmed`. Four items, each accounted for in SUMMARY.md.

1. YOU ARE REVERSING A WRITTEN DECISION, AND THE SPEC'S ARGUMENT ONLY COVERS HALF OF IT. `ELEMENTS_WITHOUT_A_ROLE`'s `dialog` entry (`src/a11y.rs:810`) reads: *"would be Role::Dialog with a required name and `modal`; gpui has no `aria_modal` builder, and a dialog that announces itself unmodal is worse than one that waits."* The spec confirms gpui still has no `aria_modal` — so the reason it was excused is intact, and this change announces anyway. For the confirmation half I accept the trade: `Role::AlertDialog` plus a scrim conveys more than silence does, and the whole point of the issue is that affordance. For the **plain titled dialog** the recorded sentence still applies unanswered — it will announce `Role::Dialog`, unmodal, which is the exact thing the module wrote down as worse than waiting. Do one of two things and say which: keep the plain dialog silent and announce only in confirm mode (the textual scan in `every_element_module_declares_a_role` forces the entry's deletion either way, so this costs you nothing structurally), or announce both and write the argument for why the recorded reason no longer holds — in `dialog.rs`, where the deleted entry's reason has to live now. What is not acceptable is deleting the sentence and replacing it with nothing.

2. THE TESTS WERE NEVER RUN, AND IN THIS REPOSITORY THEY FIT. Eight tests are type-checked and unrun, and the spec names the one most likely to fail: `a_confirmation_opens_with_the_safe_action_focused` reads `window.focused` immediately after `open` on an entity that is never drawn, and may need the real-window `Harness` pattern instead. A scout on #195, in this same repository and the same conditions, did a **cold build in 7m11s** and then ran `cargo test --lib` to 501 passed — which includes linking the test binary. So the budget premise here was wrong. Run `cargo test --lib elements::dialog` first, then `cargo test --lib`, and report both. If the focus test needs the `Harness` shape, change it rather than weakening the assertion: "the safe answer has focus when it opens" is the property this feature exists for.

3. STATE THE `destructive = true` DEFAULT WHERE A CALLER READS IT. A confirmation that nobody has thought about renders a red Confirm. The safety argument for that is real, and so is the cost — a library whose every confirmation is red teaches people that red means nothing. Keep the default if you want it, but put the argument in the `Confirmation` doc comment rather than only in the spec, so the next caller who wants a plain "Save changes?" finds `.destructive(false)` and the reason it exists.

4. THE THREE gpuikit SPECS IN FLIGHT ALL TOUCH `src/a11y.rs`, AND I INTEND TO BATCH THEM. #195 rewrites `the_focus_rule_covers_the_roles_a_keyboard_operates` exhaustively; #181 adopts `Role::Splitter` into the focus rule. Your reach into that file is the `ELEMENTS_WITHOUT_A_ROLE` entry and nothing else, so there should be no textual conflict — but do not touch the focus rule or its test, and if you find yourself needing to, stop and say so.

Checked so you do not have to: `role_requires_a_name` really does cover both `Role::Dialog` and `Role::AlertDialog` (`a11y.rs:637`), so the untitled-dialog guard is load-bearing rather than defensive; `ButtonVariant` really is a single-arm enum with the `// todo: style through ButtonVariant` above it, so the prerequisite is real work; and nothing in this repository's main consumer (`app-gpui`) calls `Button::variant` as a getter or names `ButtonVariant` at all, so the inherent-shadows-trait collision has no downstream victim today. Keep the doc note about `traits::button::Button::variant(&b)` anyway — the next caller will meet it.

The focus trap you named as out of scope is filed separately; do not widen this to include it.

## Directions for this implementation

The orchestrator agent added the following when requesting this build. It is **not** part of any spec above, and no reviewer has seen it — it is addressed to you.

Treat it as a requirement, not a suggestion. The specs are still what is being implemented; these directions say how to go about it. Where one genuinely conflicts with a spec, the direction wins — it was written after the spec was approved, with this build in view — but **say so in `SUMMARY.md`**, because the reviewer reads the spec and cannot see this section.

Account for every direction in `SUMMARY.md` — including any you decided against, and why. A direction you silently dropped is indistinguishable from one you never read.

All three of these specs edit `src/a11y.rs`, which is why they are one branch. Do them in this order and commit each separately:

1. **#195** first — the exhaustive rewrite of `the_focus_rule_covers_the_roles_a_keyboard_operates`. It replaces the test the other two then have to satisfy, so doing it first means the others are checked by it rather than racing it. All 16 arms of `role_requires_keyboard_focus`, not 4.
2. **#181** second — adopting `Role::Splitter` into the focus convention and deleting its own `tab_index`. Built on #195's rewritten test, this is a one-line change to the rule plus the deletion, and the test proves it.
3. **#165** last — the confirmation dialog. Its only reach into `a11y.rs` is deleting the `dialog` entry from `ELEMENTS_WITHOUT_A_ROLE`; it must not touch the focus rule or its test. If you find yourself needing to, stop and say so in SUMMARY.md rather than reconciling it yourself.

Run `cargo test --lib` after each of the three, not once at the end — a green run after step 1 is what tells you the rewrite is honest, and a red run after step 3 that you only see at the end is three changes to bisect instead of one. A scout measured a cold build in this repository at 7m11s and then ran the full 501-test lib suite, so budget for it rather than skipping it.

`cargo test --lib`, not `--all-targets` and not `--features examples`: Cargo.toml's own comment explains that the examples feature exists to keep eight links of gpui out of the ordinary commands.

## Your job

1. Implement every spec above, in order, as one coherent change in the cloned repo (cwd). You are on the right branch already.
2. Run the project's tests / lint / typecheck — get them green.
3. Commit your work with clear messages (a git identity is configured).
4. Write `SUMMARY.md` in the repo root: one or two paragraphs describing the change, suitable as a pull request body. Do not use GitHub closing keywords (`Closes #N`, `Fixes #N`) — the server links the issues itself.
5. Do NOT push and do NOT open a PR — the server does both.

**You have 60 minutes, once.** That is the whole run — the clone before you started, this turn, the supervisor's own test run and the packaging after it — measured on the wall clock from dispatch. There is no later: when you end your turn the run is over. A backgrounded command buys you nothing — its child is killed with the turn — so anything whose result you need must be awaited inline, and a poll loop over a file another process will write can only report to a turn that has already ended. Nor should you start what cannot finish: a cold build in a large workspace can run forty minutes, so weigh what a command will cost against what is left.

On step 2: when this project declares a test suite at `.tasks/verify`, the supervisor runs it itself after you finish, against the committed tree your branch carries. If it fails you get one chance to fix it and then the build fails with no pull request, so getting there first is entirely in your interest. It reads that script out of the build's BASE commit, so editing it changes nothing about what runs.
