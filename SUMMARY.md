# Cut the dev profile's dependency debug info so linking stops being OOM-killed

Linking any example in this repository is a whole-program link of gpui, and the
debug info was nearly all of what the linker held. It cost the same for every
example, because the bytes are gpui's and not the example's: measured here,
`popover_demo.rs` (5 KB) peaked at **2687 MB** and `showcase.rs` (152 KB) at
**2730 MB** — a 1.6% spread across a 30x spread in source size. cargo picks `-j`
from the CPU count and knows nothing about memory, so four cores meant four
linkers at that peak and the kernel killed them, reported as `ld terminated with
signal 9 [Killed]` — a message naming no crate and no symbol, which reads as a
compile error and is not one. There is no "heavy example" to fix, and the
issue's focus on `markdown_streaming` does not survive the measurement.

`[profile.dev.package."*"] debug = 0` is the whole fix: **2687 -> 818 MB** and
**2730 -> 960 MB**, a 70% and 65% cut. `[profile.dev-debuginfo]` (`inherits =
"dev"`, `debug = 2`, package table restated) is the hatch, so the answer to "but
I need a debugger" is never "edit `[profile.dev]` and remember to put it back".
Alongside it: `scripts/check.sh` runs the checks `examples/README.md` already
documented under `set -euo pipefail`, taking each cargo status from
`${PIPESTATUS[0]}` rather than `tee`'s — a piped build reporting the pipe is the
second half of the issue, and is how a killed link was once read as a green run
— and explains `signal 9` when it sees it. `[package] autoexamples = false`
plus an `[[example]]` block for the previously auto-discovered
`examples/context_menu.rs` makes the manifest the whole list. Six guard tests in
`src/build_memory_guard.rs`, in the idiom `release_version_guard` and
`release_input_validation` already use, keep all of it from quietly coming back;
they are in the lib rather than `tests/` because an integration test would be
one more whole-program link.

## The split measurement (the reviewer's required change 1)

Measured on this box (aarch64, 4 cores, 8 GB, `-j 4`), peak `ld` RSS via a
linker wrapper kept set for the whole comparison, one variable changed at a
time:

| `[profile.dev]` | `[profile.dev.package."*"]` | `popover_demo` | `showcase` |
| --- | --- | --- | --- |
| `debug = 2` | *(inherited `2`)* | 2687 MB | 2730 MB |
| `debug = 2` | **`debug = 0`** | **818 MB** | **960 MB** |
| `debug = "line-tables-only"` | `debug = 0` | 732 MB | 802 MB |

**The dependency cut alone is inside budget, so `line-tables-only` is not
shipped.** At 960 MB, four concurrent linkers want ~3.8 GB, which fits the
4-core / 5.9 GiB / no-swap box that was failing with room to spare. The
first-party cut buys a further 158 MB on `showcase` (16%) and costs locals in
the frames a gpuikit contributor is actually standing in — the reviewer's point
exactly, and the spec's own estimate of "another 100-150 MB" for the next step
down was about right. `[profile.dev] debug = 2` is therefore untouched, and
`dev-debuginfo` stays a rarity rather than a routine flag. Both numbers are in
the table above and the reasoning is in a comment on the `[profile.dev]` line,
so the next person to reconsider this profile has one number per decision.

## Review feedback

- **1. Measure the dependency cut alone and let it decide whether
  `line-tables-only` ships.** Done, before the manifest was written; numbers in
  the table above. The dependency cut alone is inside budget, so
  `line-tables-only` was **dropped** — `[profile.dev]` keeps `debug = 2`. This
  diverges from the spec's Implementation Approach, which listed
  `line-tables-only` as "the load-bearing change"; the measurement says the
  dependency tier is load-bearing and the first-party tier is 16%.
- **1b. Ruling: take the dependency cut as a repository default, with a named
  profile as the hatch.** Done as ruled: `[profile.dev.package."*"] debug = 0`
  plus `[profile.dev-debuginfo]`.
- **2. The examples guard must cover the directory form
  `examples/<name>/main.rs`, mutated the way the flat case was.** Done —
  `discoverable_example_paths()` treats a directory containing `main.rs` as a
  discoverable example, and both mutations (`examples/zzz_probe.rs` and
  `examples/zzz_probe/main.rs`) fail the test with the orphan named. The failure
  message also states which two forms are checked, and that a module in a
  subdirectory with no `main.rs` (as `examples/input/fixtures.rs` is) is
  invisible to both. Not declined, so the fallback sentence was not needed.
- **3. Say in `## Building` that these settings apply to this repository and
  name what a consumer would set.** Done in both `examples/README.md` and
  `README.md`: cargo takes profiles from the workspace root, so an app depending
  on gpuikit gets its own, and the equivalent is
  `[profile.dev.package."*"] debug = 0` in *their* manifest.
- **Recorded-so-not-re-litigated items** (`required-features`,
  `split-debuginfo`/`.cargo/config.toml`, restating the `dev-debuginfo` package
  table, `${PIPESTATUS[0]}`, the guard living in the lib, `Cargo.lock` being
  gitignored, `CARGO_TARGET_<TRIPLE>_LINKER` fingerprinting, `--all-targets`
  rebuilding gpui under `test-support`, no `[build] jobs` cap): all carried
  through as approved. Not re-argued here.
- **No clippy; leave `src/input/bindings.rs:461` alone.** Both followed. That
  warning is still the only one in the build.

## Directions from the orchestrator

- **Do the split measurement before writing the manifest.** Done in that order —
  the `debug = 0`-only manifest was built and measured first, and
  `line-tables-only` was added only to measure it and then removed.
- **Cover the directory form and mutate it with `examples/zzz_probe/main.rs`.**
  Done; see review item 2.
- **One sentence in `## Building` about consumers.** Done; see review item 3.
- **Verify with the spec's commands and report real numbers, not the spec's.**
  Done — every number in this file was measured on this box. Where the spec's
  own figures are quoted (2.7 GB, the 4-core / 5.9 GiB box) they are its
  framing, and this box independently reproduced the shape: 2687/2730 MB here
  against the spec's 2673/2771 MB.

## Verification

All run on this box with `-j 4`:

- `scripts/check.sh` (fmt, `test --lib`, `check --all-targets`,
  `check --all-targets --features editor`): **exit 0**, 66s warm.
- `cargo test --lib`: **498 passed / 0 failed** — the 492 that were there plus
  the 6 new guards.
- `scripts/check.sh --link`, which adds
  `cargo build --all-targets --features editor` (the heaviest documented
  configuration): **exit 0**. All 8 examples link, including `context_menu` now
  that it is declared rather than auto-discovered, and the peak `ld` RSS across
  the whole set was **892-991 MB** — an 11% spread over eight binaries, which is
  the "no heavy example" finding holding across the tree rather than across the
  two examples that were measured in detail.
- `cargo fmt --check`: clean.

**The guards were mutation-tested**, because a guard that cannot fail is worse
than none. Six mutations, each failing exactly the intended test with its own
message: `autoexamples = true`; a stray `examples/zzz_probe.rs`; a stray
`examples/zzz_probe/main.rs`; `[profile.dev-debuginfo] debug = 1`; dropping
`opt-level` from the `dev-debuginfo` package table; and, in `scripts/check.sh`,
`set -euo pipefail` -> `set -eu`, `${PIPESTATUS[0]}` -> `$?`, and removing the
`signal 9` branch.

That last group found a real defect in the guard as first written. `check.sh`
explains `pipefail` and `${PIPESTATUS[0]}` in its own header comment, so a
`contains()` over the whole file matched the *explanation* and stayed green when
the actual `set -euo pipefail` line was mutated to `set -eu`. The test now
strips comment lines before looking, and asserts a line whose trimmed content is
exactly `set -euo pipefail`; all three `check.sh` mutations fail as intended.

The two `[profile.dev]`-tier assertions are not mutated directly, for the reason
the spec gives: flipping `debug` there changes the effective profile and forces
a full rebuild each way. They are covered instead by the fixture test (which
proves the reader parses a `debug = 2` and a trailing comment correctly, and
that it does not match `debug-assertions` on the `debug` prefix), by the
`dev-debuginfo` mutations (which prove the same lookup fires against the real
`Cargo.toml`), and by a vacuity assertion that takes the value parsed out of the
fixture's expensive profile and asserts `CHEAP_DEBUG` rejects it.

## Notes

- Peak RSS was measured with a linker wrapper set via
  `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER`, which execs `cc` with the
  same arguments and records `RUSAGE_CHILDREN`. Per the spec's pitfall it was
  kept set across the whole comparison, since unsetting it invalidates every
  linked unit and cascades into a full rebuild. It is a measurement harness
  only — nothing about it is committed.
- The spec's box was 4-core / 5.9 GiB / no-swap; this one is 4-core / 8 GB, so
  the OOM kill itself could not be reproduced here. What is reproduced is the
  quantity that causes it — peak `ld` RSS, before and after — and it lands
  within 2% of the spec's figures.
- **Residual risk, deliberately not addressed:** ~960 MB per link still times
  the job count, so a many-core runner with little RAM would still be short.
  `CARGO_BUILD_JOBS=1` is documented in `scripts/check.sh --help`, in
  `README.md` and in `examples/README.md`, and the script says so itself when it
  sees a `signal 9`. Nothing caps `-j`, because a `.cargo/config.toml`
  `[build] jobs` would serialise every build for every contributor.
