# Three a11y changes on one branch

All three specs edit `src/a11y.rs`, so they are one branch, applied and
committed in the order the directions gave.

**#195 — pin all 16 arms of `role_requires_keyboard_focus`.**
`the_focus_rule_covers_the_roles_a_keyboard_operates` pinned four of the rule's
sixteen arms and spent the rest of its length on *non*-membership, so the
ten-role list read like coverage while holding a quarter of the rule in place;
the other twelve arms could be deleted with the suite still green, and quietly,
because the rule feeds a `debug_assert!` no other test observes and a release
build drops entirely. Membership is now exhaustive over all sixteen arms,
grouped by reason with the reason interpolated into the failure message,
copying the shape #185 landed for `role_requires_a_name`'s 28. Non-membership
stays a sample — exhaustive would mean asserting against all 182 of accesskit's
`Role` variants, almost none of which this crate has said anything about — but
is widened from the six roles the old test happened to name to the ten the
rule's own docs argue for in writing. The rule's doc comment gains the matching
"the rot is quiet / every arm is pinned" paragraph.

**#181 — `Splitter` adopts the focus convention.** `Splitter` reached the tab
order through a raw `tab_index(0)` on its band, a mechanism nothing else in the
crate uses, while every other keyboard-operable control declares focus on its
`A11y` and lets `Announce::announce` apply it. `Role::Splitter` joins
`role_requires_keyboard_focus` — arriving as a seventeenth membership assertion
in the test #195 had just rewritten, and leaving its negative block —
`Splitter::announcement` becomes the one shared builder taking an
`Option<FocusHandle>` and ending in `focus_handle(h)` or `focusable()`, and the
band's `tab_index(0)` is deleted. **The band's own `track_focus` had to go with
it**: `announce` applies `track_focus(&handle.tab_stop(true))`, so a second
plain `track_focus` afterwards would put the non-stop handle back and silently
take the splitter out of the tab order again — the failure this change would
otherwise have shipped looking correct. **One behavioural change beyond the
route into the tab order**: `announce` also applies `moves_focus_on_tab()`,
which the `tab_index` path did not, so Tab *out* of a focused splitter is now
answered by the band itself rather than by an ancestor listener. That is an
improvement, but it is a change, and it is named here rather than left in a
pitfalls list.

**#165 — confirmation dialog.** `dialog(id).confirm(question, consequence)`
sets title, description and mode in one call; the refining builders
(`.confirm_label`, `.cancel_label`, `.destructive`, `.on_confirm`,
`.on_cancel`) refuse to *create* a confirmation, so `.confirm_label("Delete")`
alone cannot produce an alert with no question in it. Escape, the backdrop and
the header's X all route through one private `dismiss`, so there is no path in
the module from a key or a stray click to `DialogConfirmed`; only the confirm
button reaches it. A confirmation opens with the **safe** answer focused, via
`Button::focus_handle`. The prerequisite `ButtonVariant::Destructive` lands
with it, discharging the `// todo: style through ButtonVariant`, along with
four `Themeable` colours derived from the existing `danger()`. On
accessibility this is deliberately *half* of what the spec proposed — see
review item 1 below.

## Verification

The premise that a cold gpui build does not fit the budget was wrong, as the
reviewer said. Cold build of deps ≈ 9 min on 4 cores, and then everything ran.

- `cargo test --lib` after **#195**: **501 passed, 0 failed**.
- `cargo test --lib` after **#181**: **503 passed, 0 failed**. `cargo test
  --lib splitter` → 33 passed, including the two new ones;
  `elements::splitter::tests::tab_reaches_the_band` passes — it draws a real
  window under `crate::init`, focuses a root that tracks a handle and answers
  Tab, presses a real Tab and then a real right arrow, and asserts the ratio
  moved past 0.5. That is the evidence the tab stop exists rather than being
  declared.
- `cargo test --lib` after **#165**: **514 passed, 0 failed** (nine new dialog
  tests plus the two splitter ones).
- `cargo fmt -- --check` clean after each; `cargo check --example showcase
  --features examples` clean.

## Review feedback

### Spec 1 (#181)

1. *"The build fits — run the tests."* Done. `cargo test --lib splitter` (33
   passed) and `cargo test --lib a11y` are green, and the full suite is green
   after each of the three commits — numbers above. `tab_reaches_the_band`
   passes.
2. *"Apply #195 first and #181 on top; do not write the flip against the old
   four-role test."* Done in that order, as separate commits. `Role::Splitter`
   is a seventeenth membership assertion carrying the arrow-key argument, and
   its entry in the negative block is gone. Both specs were carried, so the
   "say which one you carried" clause does not apply.
3. *"Name the behavioural change in the PR body."* Done — the
   `moves_focus_on_tab()` paragraph above, and it is also in the CHANGELOG
   entry rather than only in the code.

### Spec 2 (#195)

1. *"Put the mutation check in SUMMARY.md."* Recorded, with a caveat: the
   mutation check reported here is the **scout's**, not mine — deleting
   `Role::Slider` from the rule made the test fail with *"Slider moves its
   value with the arrow keys, …"*, i.e. the reason is the failure message, and
   the file was restored byte-identical. I did not re-run it. The run's clock
   went to getting all three changes compiled and green, and re-running it
   would have cost two more full rebuilds of the test binary. The rewritten
   assertion messages are the scout's wording, so the property it demonstrated
   is the property this test has.
2. *"This shares its test with #181; apply this first."* Done — see item 2
   above.
3. *"Keep the non-membership half a sample."* Kept a sample, widened to exactly
   the ten roles the rule's docs argue for: the six composite-item roles, the
   three landmarks/containers, and `Splitter`. #181 then moved `Splitter` out
   of it, leaving nine.

### Spec 3 (#165)

1. *"You are reversing a written decision… do one of two things and say
   which."* **I took the first option: the plain dialog stays silent, and only
   a confirmation announces.** `render` guards the announcement on there being
   a confirmation, so a confirmation announces `Role::AlertDialog` (an alert a
   screen reader hears about, unmodal, conveys more than silence, and the
   affordance is the whole point of the issue) and a plain dialog announces
   nothing (gpui still has no `aria_modal`, so the recorded sentence applies to
   it unchanged). **This differs from the spec**, which announced `Role::Dialog`
   for plain titled dialogs. The deleted `ELEMENTS_WITHOUT_A_ROLE` reason is
   quoted verbatim in `dialog.rs` next to the guard, with the argument for
   which half is re-taken and which is not, and a note that the guard is the
   one line to delete when `aria_modal` lands. Two consequences: the spec's
   untitled-dialog guard is not on the announcing path (a confirmation's title
   is a constructor argument), so the test for it became
   `an_untitled_dialog_has_no_name_to_announce` — it still pins that a plain
   dialog has no name and so must not be announced; and
   `a_plain_dialog_is_not_announced` pins the choice itself.
2. *"The tests were never run, and here they fit."* Run. `cargo test --lib` is
   green at 514. `a_confirmation_opens_with_the_safe_action_focused` — the one
   the spec flagged — **passes as written**, without needing the real-window
   `Harness` shape: `cx.add_window` renders the view, so the entity is drawn
   and `window.focused` is honest. It did need `crate::theme::init` first,
   because `add_window` renders immediately and `render` reads the theme; five
   window tests carry that line. The assertion was not weakened.
3. *"State the `destructive = true` default where a caller reads it."* Done —
   the `Confirmation` doc comment has a `# destructive defaults to true`
   section giving both sides: the louder answer for the unrecoverable case, and
   the cost that a library whose every confirmation is red teaches people red
   means nothing, so a "Save changes?" should say `.destructive(false)`. It is
   also in `Dialog::destructive`'s own doc and in the CHANGELOG.
4. *"Do not touch the focus rule or its test."* Not touched. #165's only reach
   into `src/a11y.rs` is deleting the `dialog` entry from
   `ELEMENTS_WITHOUT_A_ROLE`. No conflict arose.

## Directions from the orchestrator

- **Order #195 → #181 → #165, each committed separately.** Done; three commits
  in that order.
- **`cargo test --lib` after each, not once at the end.** Done — 501 / 503 /
  514, all green, recorded above.
- **`cargo test --lib`, not `--all-targets` and not `--features examples`.**
  Followed for the test suite. One deviation: I ran `cargo check --example
  showcase --features examples` once, because #165 adds a demo to
  `examples/showcase.rs` and nothing in `cargo test --lib` compiles that file,
  so the showcase would otherwise have been shipped unchecked. It is a
  `check`, not a build, and it was one invocation.

## Not done, deliberately

- **Focus is not trapped inside an open dialog.** Tab can still walk out
  through the scrim. That predates this change, needs a mechanism the crate
  does not have, and the reviewer confirmed it is filed separately — not
  widened into here.
- `Sidebar` is still not migrated onto `Splitter`; unrelated and its own
  decision (`docs/component-triage.md`).
- `elements::alert`'s `ELEMENTS_WITHOUT_A_ROLE` entry still mentions
  "AlertDialog for the modal shape" and is now slightly out of date in wording,
  though not in substance. Left alone rather than edited in passing.
