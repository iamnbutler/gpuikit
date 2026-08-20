# Stop `ld` being OOM-killed while linking gpuikit's examples

Eight example binaries live in `examples/`, and each one is a full link of gpui.
`cargo build --all-targets` and a bare `cargo test` link all eight; cargo sizes `-j`
from the CPU count with no knowledge of a memory limit, so several `ld` processes run
at once — each holding that binary's debug info, which `[profile.dev] debug = 2` made
maximal — and the kernel kills one. The message, `ld terminated with signal 9 [Killed]`,
names no crate and no symbol, so it reads as a compile error that does not exist; three
separate runs in one week were lost hunting a type error that was never there.

This is build configuration only — no library code, no API change, no dependency change.
Three settings: `[profile.dev]` now sets `debug = "line-tables-only"`, which is the
biggest lever because it reaches gpui too (`[profile.dev.package."*"]` sets only
`opt-level`, so it does not hold debug info back); a new `.cargo/config.toml` passes
`-Csplit-debuginfo=unpacked` on the two Linux targets, where the dev default of `off`
copies every byte of DWARF through the linker into the image; and every `[[example]]`
now carries `required-features = ["examples"]` against a feature that enables nothing,
which removes the eight links from `cargo test` and `cargo {build,check} --all-targets`
outright. `examples/context_menu.rs` is declared in `Cargo.toml` for the first time —
it was autodiscovered, and an autodiscovered target cannot carry `required-features`, so
it would have been the one example still linked. `split-debuginfo` is deliberately *not*
a `[profile.dev]` key: cargo profiles take no `cfg` and the value is a hard error on
`windows-msvc`. The costs, all stated in `CHANGELOG.md` and `examples/README.md`:
backtraces keep file and line but lose the type and variable detail a debugger wants
(`RUSTFLAGS="-Cdebuginfo=2"` restores it for one build), running an example now needs
`--features examples`, and `cargo check --all-targets` no longer type-checks the
examples without it — `check` never links, so the documented command now passes it.
`src/build_profile_guard.rs` (new, `#[cfg(test)]`, following the idiom of
`src/release_version_guard.rs`) holds the three settings in place under `cargo test
--lib`, and one of its tests reads `examples/` from disk, because one new undeclared
file there restores the whole bug.

## The coverage gap, in the terms that decide whether #180 can be closed

**`cargo test --all-features` still links all eight examples.** Cargo has no way to
exclude a feature from `--all-features`, so the gate does not apply to it — and that is
the command that produced the kill on `markdown_streaming` in report #152, the first of
the three runs this issue was filed from. The gate therefore does **not** cover the
originally reported trigger; only the debug-info reduction does. That reduction is
measured below — 2.67 GiB of peak `ld` RSS down to 0.94 GiB, so four concurrent links go
from ~10.7 GiB to ~3.7 GiB — but whether that is enough on any particular box is a
question about that box's memory, not one this branch can answer. A reader who still sees an OOM under
`cargo test --all-features` after this lands should treat that as expected of this
change, not as a regression.

The next moves, written down here and in the changelog so they are not rediscovered:

1. **`[profile.dev.package."*"] debug = 0`.** Dependency debug info is the bulk.
   `build_profile_guard.rs`'s override test rejects only a *raised* value, so this stays
   open without editing a test.
2. **Move the examples into their own package** (a workspace member depending on
   `gpuikit`). It is the only arrangement in which *no* invocation of this library's own
   cargo commands builds them — `--all-features` included. Not implemented here; it is a
   layout change well outside this fix's scope.

## What was verified, and what was not

- `cargo test --lib` on this branch: **501 passed, 0 failed**, including all nine of
  `build_profile_guard`'s tests. (Nothing in this run ran a bare `cargo test` or an
  `--all-targets` build, which are the commands that link the examples.) Five mutations — `debug = 2`, a dropped
  `required-features`, a new undeclared `examples/newthing.rs`, `split-debuginfo=off`,
  and a README command losing `--features examples` — each fail exactly one test and no
  others.
- `cargo metadata --no-deps` on the real manifest: all eight examples carry
  `required-features: ["examples"]`, the feature list is as intended, and cargo accepts
  the profile value (it validates these, so this is a real check).
- `rustfmt --check` clean on both changed Rust files.
- **Measured, on a 4-core aarch64 Linux box with 8 GB of RAM** — the reviewer's
  before/after, taken by sampling every process's RSS at 5 Hz while
  `cargo build --example showcase --features examples` ran, and reporting the peak seen
  for `ld`. "Before" is `debug = 2` with no `.cargo/config.toml`, built into a separate
  target directory so the two trees do not share artifacts:

  | | peak `ld` RSS | linked `showcase` |
  | --- | --- | --- |
  | before | 2.67 GiB (2,800,616 KB) | 764 MB |
  | after | **0.94 GiB** (982,072 KB) | 146 MB |

  2.85× less memory in the linker, and a linked image 5.2× smaller with the DWARF now
  in `.dwo` files beside it. The number that matters is what cargo does with it: at the
  default `-j` on 4 cores, four of these run at once — about **10.7 GiB before, 3.7 GiB
  after**. That is the difference between not fitting in the ~5 GB the reported runs had
  and fitting with room, and it is why the gate (which removes the links entirely from
  the common commands) and the debug-info reduction (which is all `--all-features` has)
  are both here.

## Review feedback

1. **State the coverage gap where the reviewer will see it.** Done — its own section
   above, in those terms: `--all-features` still links all eight, the gate does not cover
   the trigger from #152, only the debug-info reduction does, and its magnitude is
   unmeasured.
2. **Name the alternative the spec did not consider (own package), plus the spec's own
   next lever, in `SUMMARY.md` and the changelog.** Done in both, as an ordered list of
   next moves. Not implemented, as instructed.
3. **Drop or weaken the eighth guard test.** Weakened, not dropped: it now asserts only
   that every `cargo run/build --example` command in `examples/README.md` passes
   `--features examples` — commands, never prose. It no longer looks at the `pipefail`
   explanation or at any wording. The `pipefail` section itself stays in
   `examples/README.md` (the spec asked for it, and it is true for anyone building here),
   but nothing tests it, and I have not touched the agent-prompt half being fixed in the
   Tasks repo.
4. **Take the RSS measurement if the budget allows.** Taken: peak `ld` RSS for
   `cargo build --example showcase --features examples` fell from 2.67 GiB to 0.94 GiB,
   with the full method and the four-concurrent-links arithmetic in the section above.
   One deviation from the reviewer's wording: the builds ran at `-j 4` rather than
   `-j 1`. Only one example is linked either way, so the `ld` process being sampled is
   alone in both runs and the number is the same; `-j 1` would only have serialised the
   ~500 dependency compiles ahead of it, which did not fit.

## Directions from the orchestrator (no reviewer saw these)

- *"Run `cargo test --lib` yourself; do not run bare `cargo test` or `--all-targets`."*
  Followed. `cargo test --lib` was run against a cold registry and passed 501/501; no
  bare `cargo test` and no `--all-targets` build was run at any point. The mutation
  results above come from running the guard module standalone via `rustc --test` with
  `CARGO_MANIFEST_DIR` set — it depends on nothing in the crate, so that is the whole
  file under test, and it is seconds per mutation rather than a rebuild.

No item was dropped, and nothing here conflicts with the spec except item 3, which the
spec's eighth test would otherwise have required verbatim; the feedback wins, as
instructed, and the weakened form is described above.
