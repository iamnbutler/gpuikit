# Pin every arm of `role_requires_a_name` with a per-role test

`role_requires_a_name` feeds a `debug_assert!` rather than a value an element
reads back, so its rot is quiet: nothing held the list's contents in place, and
deleting `| Role::DefaultButton` from it left the entire lib suite green. This
adds `the_name_rule_covers_the_roles_that_are_nothing_without_one` beside the
existing `the_focus_rule_covers_the_roles_a_keyboard_operates`, pinning **all 28
arms** of the list in seven families — single controls, composite items, value
controls, `Splitter`, text entry and choosers, dialogs, `Image` (6+6+4+1+8+2+1) —
with each family's reason as the assertion's failure message and `{role:?}`
interpolated, so a role that falls off is named alongside the argument for it
having been there. Three absences are pinned too: `Role::Complementary` and
`Role::Document`, cited by the rule's own doc comment, and `Role::ListBox`, whose
exclusion `src/elements/select.rs` argues in writing. `role_requires_a_name`'s doc
comment gains a paragraph pointing at the test; it uses plain backticks rather
than an intra-doc link because the `tests` module is `#[cfg(test)]`, the same form
`A11y::active_descendant` already uses.

`the_naming_rule_covers_the_roles_that_name_themselves` was half list and half
behaviour, so it is split the way the focus rule already is: the list half is
subsumed by the new test, and the behavioural half is renamed
`a_required_name_is_absent_blank_or_given`. `src/a11y.rs` is the only file
changed (+131 / −7). The test's doc comment states the one asymmetry worth
knowing: membership is exhaustive, non-membership is not — adding some *other*
role to the list still passes, which is deliberate, because an assertion with no
argument behind it is the failure mode this module is organised against.
`Role::Splitter` is now pinned in two places (here and
`src/elements/splitter.rs:1017`), which is intended — the element asserts the rule
it depends on. The focus rule is untouched: its own coverage gap is
iamnbutler/gpuikit#195 and is out of scope here.

## Verification

Every number below was produced by running the thing, in the foreground, on this
branch.

- **Suite delta is exactly +1.** Baseline on this branch before the change:
  `test result: ok. 642 passed; 0 failed`. After: `test result: ok. 643 passed; 0
  failed`. One test removed, two added. (The spec's 622/623 are stale — #192,
  #193 and #194 landed under it, as the build directions noted; the delta is what
  matches.)
- **The spec's control repro, post-change:** deleting `| Role::DefaultButton`
  from the list now fails at `src/a11y.rs:1349` with
  `DefaultButton is a single control whose whole announcement is name + role +
  state, …` printed.
- **Full mutation sweep**, run per mutant as
  `cargo test --all-features --lib the_name_rule_covers` so every kill is credited
  to the new test alone rather than to `splitter.rs`'s own assertion:
  **28/28 arm deletions KILLED**, one per role; **3/3 exclusion reversals KILLED**
  (adding `Complementary`, `Document`, `ListBox`); **1 survivor — adding
  `Role::Group` — expected, documented in the test's doc comment, and left alive
  deliberately.** `src/a11y.rs` was byte-compared against its pre-sweep copy
  afterwards to confirm no mutant leaked into the commit.
- `cargo fmt --check` clean.
- `cargo clippy --all-features --lib --tests` introduces no warning mentioning
  `a11y.rs`; the 42 warnings it reports are elsewhere in the crate and
  pre-existing, including the `unused_mut` at `src/input/bindings.rs:461`.

## Review feedback

1. **Prove "no assertion is lost" rather than asserting it — counts, both sides.**
   Done. The old `the_naming_rule_covers_the_roles_that_name_themselves` contained
   **10** `assert!`/`assert_eq!` calls: 4 list and 6 behavioural. The two tests
   that replace it contain **15**: 9 in the membership test and 6 in the
   behavioural one. The 9 are not 9 role checks — 5 of them are `for role in […]`
   loops, so they cover **31** role checks (28 arms + 3 exclusions), and the 4
   roles the old list assertions named (`Button`, `CheckBox`, `Complementary`,
   `Document`) are all among them. The 6 behavioural assertions **are** verbatim:
   `diff` of the old body against the new one, from the first assertion to the
   last, reports them identical. Nothing was touched.
2. **Do not repeat the mislabelled framing of the focus sweep, and do not widen
   this change to the focus rule.** Done — no sentence about which focus-rule arms
   were killed or survived appears in the commit message, the code, or this
   summary, and `role_requires_keyboard_focus` and its test are byte-identical to
   `main`. The follow-up is #195.
3. **Do not run `cargo clippy --all-features --all-targets`.** Not run. The form
   run was `cargo clippy --all-features --lib --tests`, reported above.
4. **Run tests in the foreground; read the `test result:` line out of that
   command's own output; build no watcher.** Done. Every `cargo test` invocation
   here ran in the foreground and every count quoted above was read from that
   command's own stdout. Nothing was backgrounded and no watcher was created. The
   line I went by is `test result: ok. 643 passed; 0 failed; 0 ignored; 0
   measured; 0 filtered out` — the process exit code was ignored, since gpuikit
   #190's `async-io` teardown abort makes it unreliable on a green suite.

## Build directions

- **`main` moved; check the delta, not the absolute.** Done — baseline 642, after
  643, exactly +1, as recorded above. The three landed PRs touch nothing in
  `src/a11y.rs`, so nothing in the spec was invalidated.
- **Rely on the verified line numbers and the `ListBox` citation rather than
  re-deriving them.** Taken as given and confirmed in passing while editing:
  `role_requires_a_name` at line 597 with 28 arms, the seven families summing to
  exactly those 28.
- **The focus-rule gap is #195 and is not mine.** Not widened; see feedback item 2.
- **Foreground the tests.** See feedback item 4.

Nothing in the review feedback or the build directions conflicted with the spec,
so there is no conflict to flag. One departure is the spec's own and is restated
here because it is deliberate: the *shape* of the focus rule's test is mirrored
exactly, but the *coverage* is not — the focus test samples 4 of its arms, this
one is exhaustive over all 28, because a sample would have left the reported
defect in place for 24 roles.

The cold first build of this checkout took roughly 12 minutes of compilation
before any test could run; incremental cycles after that are ~1s, which is what
made a 31-mutant sweep cheap enough to run rather than reason about.

Verification: PASSED — `cargo test --all-features --lib` (643 passed, 0 failed), plus `cargo fmt --check` and `cargo clippy --all-features --lib --tests`
