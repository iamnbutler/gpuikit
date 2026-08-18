# Build: #152, #158, #171

Three approved specs, landed as one change because two of them edit the same
three files (`CHANGELOG.md`, `src/elements.rs`, `docs/component-triage.md`).

**#158 adds `gpuikit::elements::splitter`** — a draggable divider between two
panes, which is `docs/issues/resizable.md`'s answer to #59's "Resizable" and
deliberately not a pane tree. The split ratio stays the caller's: the element
takes the current one, emits the new one through `on_resize`, and stores nothing
about where the boundary sits, so a layout can still be persisted, restored or
reset from outside. The drawn line is `Separator`'s existing 1px hairline; the
*interactive* band around it is 6/8/12px off the rung, which is the difference
between a divider that feels good and one nobody can hit. The band reports
`Role::Splitter` with its position, both floors and one arrow key's worth as
percentages, and follows the WAI-ARIA window splitter keyboard contract. The
drag is registered on the **window** from a `canvas` paint closure rather than
on the band, because `div().on_mouse_move` only fires while its hitbox is
hovered and a drag built on it dies the moment the pointer leaves the band —
`src/elements/slider.rs` is built exactly that way and has that bug today; this
element does not copy it and does not fix it either. 31 tests, mutation-checked
(breaking `clamp` fails 6, dropping the grab offset fails 3, removing the
unpressed-move guard fails 1, not subtracting the band from the usable space
fails 16).

**#152 narrows two claims the repository made about itself.**
`every_control_on_a_row_is_the_same_height` measured nine controls and was named
for the whole crate; it becomes `every_sized_control_on_a_row_is_the_same_height`
and now names both exclusion sets — the six elements that are not on the shared
size scale at all (`ToggleGroup`, `Tabs`, `Alert`, whose height is padding plus a
line box; `Slider`, `Progress`, `RadioGroup`, which hard-code a track or glyph
size) and the seven that *are* `ControlSized` and still are not measured
(`Field`, `Input`, `Textarea`, `Table`, `Sidebar`, `SidebarTrigger`,
`CheckboxBox`) — each with its one-line reason. And `docs/component-triage.md`
gains an attribution section saying its eleven rejections are one agent's
reading, **proposed and not ratified**, without softening a single reason; the
missing thing was attribution, not doubt.

**#171 makes the release workflow refuse a version `CHANGELOG.md` does not
name.** One new step in `release.yml` calling
`.github/scripts/verify-release-version.sh`, no new inputs, and no `if:` so a
dry run reports a wrong version instead of staying silent, plus a separable
tag-already-exists refusal. The two ways `release.yml` computes a version
stopped agreeing when #170 landed 0.8.0 *into* `Cargo.toml` as a prepared
release, and this repository has already lost a version to exactly that: its
tags are `v0.3.0 v0.4.0 v0.5.0 v0.5.1 v0.7.0`, with no `v0.6.0` though the
changelog has a heading for it.

## Review feedback

**#152 · 1 — the `Closing note for #59` section must not be merely gated.**
Done, and I took the *removal* option: the section is replaced by a past-tense
"What became of #59" that states #59 was closed `COMPLETED` on 2026-03-25, five
months before the document existed, and carries no counts at all. Removal over
correction because the block's only remaining function was to be pasted, and
there is nothing to paste it on; the paragraph that replaces it records that the
draft existed and why it is gone, so the history is not lost. A new test,
`the_59_section_does_not_promise_an_action_nobody_can_take`, fails if a
"Ready to paste" block comes back or the date goes missing.

**#152 · 2 — the renamed test's comment must name both exclusion sets.** Done.
The comment names the six unsized elements split by *why* they are off the scale,
and the seven `ControlSized` implementors that are not in the row: `Field` and
`Input` (reached through the `text-field` entry, which is `TextField` wrapping an
`Input`), `CheckboxBox` (an internal sub-part reached through `Checkbox`),
`Textarea` (multi-line, so it has no single row height to agree with), and
`Table` / `Sidebar` / `SidebarTrigger` (containers rather than row controls, with
`SidebarTrigger` flagged as the arguable one). It states the sixteen-versus-nine
arithmetic explicitly, so a reader who greps `impl ControlSized` finds the gap
already accounted for.

**#158 · 1 — keep `SplitterGeometry` and `SplitterMetrics` crate-private.**
Done: both are `pub(crate)`, kept as separate testable types exactly as designed,
each with a doc comment saying it is `pub(crate)` because the maths is easier to
check without a window rather than because a caller needs it. Promotion to `pub`
is available the moment a caller wants it, and is additive.

**#158 · 2 — the triage row stays named `Resizable`, and the literal
`docs/issues/resizable.md` must survive in the document.** Done both. The row is
`| Resizable | Shipped | \`src/elements/splitter.rs\` |`. `docs/issues/resizable.md`
is kept (deleting it would also drop `every_written_issue_is_reachable_from_the_triage`'s
count to 9, below its floor of 10) and gains a "Shipped" banner; the literal path
appears twice in the triage — in the new "One row, another name" section and in
the discharged dependency-graph edge.

**#158 · 3 — do not run `--all-targets --all-features`; never read a link result
through a pipe.** Followed. I never ran the two together. `cargo test --lib`,
`cargo test --lib --all-features` and `cargo test --doc` were each run with
output redirected to a file and the exit code read from the command itself, not
through a pipe. `cargo clippy --all-targets` was run — it only *checks* the
examples and never links them, so it carries none of the OOM risk — and the
`showcase` example was type-checked with `cargo check --example showcase`, which
I verified was not a no-op by planting a deliberate type error and watching it
fail. No example was ever linked.

**#171 · 1 — no versioned `## [x.y.z]` heading in `CHANGELOG.md`.** Followed for
all three changes: every bullet is under `## [Unreleased]`, and the topmost
versioned heading is still `## [0.8.0]`, matching `Cargo.toml`.
`changelog_names_the_version_cargo_toml_is_prepared_to_release` passes.

**#171 · 2 — run `cargo test --lib` and `cargo clippy`.** Done, and this closes
the gap that spec named. `cargo test --lib` → **418 passed, 0 failed** (of which
6 are the new `release_version_guard` module). `cargo clippy --lib --all-targets`
→ **0 errors, 0 warnings in any file this change touches**; the 29 lib warnings
and the example warnings it emits are all pre-existing (`type_complexity`,
`redundant_closure`, `needless_borrow` and friends, untouched here).

**#171 · "what you missed" — the missing `v0.6.0`.** Verified against this
checkout's tag list and the remote's, and it holds: `v0.3.0 v0.4.0 v0.5.0 v0.5.1
v0.7.0`, no `v0.6.0`, and `CHANGELOG.md` has a `## [0.6.0] - 2026-08-14`
heading. It is now recorded in both the script's header comment and the
`CHANGELOG.md` bullet, and in `src/release_version_guard.rs`'s module docs.

## Orchestrator directions

**Bullets under `## [Unreleased]` only.** Followed — see #171 · 1 above.

**The verdict counts land at 12/6/11, once.** Followed. `EXPECTED` in
`src/elements.rs` is `[("Shipped", 12), ("Issue", 6), ("Rejected", 11)]`, the
verdict table matches, and every prose statement does too: "Twelve rows below are
Shipped", "six surviving components", "12 Shipped / 6 Issue / 11 Rejected" in
"What keeps this honest", and the three counts in the new attribution section.
**This resolves against the review feedback on #152, which quotes 11/7/11** —
that was today's value on `main`, not the one to leave behind, and the direction
says so explicitly. The new attribution test checks its counts against `EXPECTED`
rather than against literals, so the two cannot drift apart again.

**Two test hazards.** Both handled: the literal `docs/issues/resizable.md`
survives in the document (twice), and the verdict row keeps the name `Resizable`.
I also found a third that neither document names — deleting
`docs/issues/resizable.md` would have taken the issue-file count to 9, below
`every_written_issue_is_reachable_from_the_triage`'s floor of 10 — which is a
second, independent reason the file had to be kept. I updated that floor's
comment, which was stale, to say why an issue body is kept after its component
ships.

**Do not run `cargo test --all-targets --all-features`.** Followed — see
#158 · 3 above.

**Do not end the turn waiting on a background command.** Followed. One
dependency-warming `cargo build --lib` ran in the background early on while I
read source files; I polled it in the foreground and it finished before any test
run. Every `cargo test` / `clippy` / `check` invocation was foreground and
completed inside its timeout. Nothing is still running.

## Deliberately not done

- `src/elements/slider.rs`'s live `on_mouse_move` drag bug is described in
  `splitter.rs`'s module comments and left unfixed; the reviewer is filing it.
- `release-deploy.yml` is untouched. It is the file that runs `cargo publish`
  and it has a second door (a pushed `v*` tag), but calling the script from
  there would break every legitimate recovery dispatch: that job checks out
  `ref: <tag>`, and `.github/scripts/` does not exist at `v0.7.0` or older.
- The pre-existing shell interpolation at `release.yml:72,74` is left alone. The
  new step interpolates only the already-computed, quoted
  `steps.bump.outputs.version`.
- `version_type` is left in place, failing loudly. It is precisely the input that
  skips a version.
- The six controls that are off the size scale are not added to it, no rejection
  is reopened or softened, `Sidebar` is not migrated onto `Splitter`, and nothing
  is posted to or closed on any issue.

## Verification

Commands run, each read from its own exit code rather than through a pipe:

| Command | Result |
|---|---|
| `cargo test --lib` | **418 passed, 0 failed** |
| `cargo test --lib --all-features` | **569 passed, 0 failed** |
| `cargo test --doc` | 2 passed, 0 failed, 51 ignored |
| `cargo fmt --check` | clean |
| `cargo clippy --lib --all-targets` | 0 errors; every warning pre-existing, none in a touched file |
| `cargo doc --no-deps` | 8 warnings, all pre-existing; none in a new file |
| `cargo check --example showcase` | clean (verified non-vacuous with a planted error) |
| `.github/scripts/verify-release-version.sh` over 15 input cases | 15/15 as expected |
| the new workflow step simulated under `bash` with `${{ }}` substituted | `custom_version: 0.8.0` passes; `patch`/`minor`/`major` all abort; the tag refusal fires on `v0.7.0` and not on an unused tag |
| both workflows parsed with `js-yaml` | the guard is step **7 of 13**, with no `if:` |

Verification: PASSED — `cargo test --lib` (418 passed, 0 failed), `cargo test --lib --all-features` (569 passed, 0 failed), `cargo test --doc` (2 passed, 51 ignored), `cargo fmt --check` clean, `cargo clippy --lib --all-targets` with no new warnings
