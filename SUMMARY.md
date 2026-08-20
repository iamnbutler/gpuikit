# Make this repository's test suite something a machine can rely on

Two changes, in the order they have to happen: first fix the flake, then declare
the gate over it.

**#190 — the `SIGABRT` at teardown.** `cargo test --lib` aborted roughly one run
in three, during teardown, after every test had already passed: a thread named
`async-io`, under `std::thread::lifecycle::spawn_unchecked`, in no test at all,
panicking in a destructor during cleanup. The crate in that name appears nowhere
in this repository, which is why the thread read as gpui's. It was not.
`gpuikit` declared a direct `smol = "2.0"` dependency, and `smol::Timer` *is*
`async_io::Timer`: constructing one reaches `async_io::driver::init`, which
spawns the process-global `async-io` OS thread, whose `main_loop` is an infinite
loop with no exit path by design. Two `Timer::after` calls — the cursor blink
interval and the toast auto-dismiss — were the only reason that thread existed
in this crate. Both now use gpui's own `BackgroundExecutor::timer`, and `smol`
is gone from `Cargo.toml`, which turns the invariant from a convention into a
compile error: `use smol::Timer` no longer builds. `src/undying_thread_guard.rs`
(`#[cfg(test)]`, in the idiom of `release_version_guard` and
`build_profile_guard`) covers what a build error cannot — a re-added manifest
entry, a `smol::`/`async_io::` path anywhere under `src/`, and the two call
sites still scheduling on the executor. `scripts/run-tests.sh` and
`docs/running-tests.md` judge a run from the harness's own `test result:` lines
rather than from an exit status, in both directions.

**#198 — the missing verification suite.** `.tasks/verify` now exists, running
`cargo test --lib`. The narrowness is the point: `--all-features` re-enables the
no-op `examples` feature (cargo offers no way to hold a feature back from that
flag) and links eight copies of gpui, which is the OOM kill of #180, and a bare
`cargo test` still links one binary per doctest — about 71 of them. Either would
fail every build in the VM instead of reporting on it. `--lib` is also what
`examples/README.md` already documents for a constrained environment and what
the build-config guards are written to run under; with no `tests/` directory it
skips nothing but doctests. The script is `set -eu`, a comment header carrying
the reasoning, and `exec cargo test --lib "$@"` — no `cd`, no pipeline, nothing
outside POSIX. This build itself still reports `Undeclared`, because the
supervisor reads `.tasks/verify` out of the base commit; the gate takes effect
for the build that stacks on this branch.

**What was and was not demonstrated.** The abort was never reproduced in this
work. It was reported on aarch64 macOS — 2 aborts in 6 runs, on both branches —
and every run behind this change was on Linux, where it did not appear once,
including a green 514-test run on the base commit. "The abort stops happening"
is therefore an argument from the mechanism plus the dependency graph, not an
observation. Each link is separately checkable: `smol::Timer` is
`async_io::Timer`; `Timer::new` reaches `driver::init`; `init` spawns the
thread; `main_loop` never returns. The check that would settle it is `cargo test
--lib` six times on an aarch64 Mac. On macOS this removes `async-io` from the
graph outright; on Linux it does not. `cargo tree -i smol` on this branch shows
exactly one path — `smol → gpui-linux → gpui-platform → gpuikit` — with the
direct edge gone, and `strings target/debug/deps/gpuikit-* | grep -c async-io`
on the built test binary is **7**, not 0 as the spec reported from its own run.
So on Linux the residue is real and reaches the binary through `gpui-platform`,
outside this crate's control. That does not weaken the fix — this crate no
longer *constructs* a `Timer`, which is what spawns the thread — but it is why
`scripts/run-tests.sh` is worth keeping rather than deleting as now-dead, and
why the false-pass arm the reviewer asked for matters.

**One behaviour change to carry forward.** This is not a pure refactor. Under
`#[gpui::test]`, the blink and toast timers now run on gpui's deterministic
clock instead of a real wall-clock thread. A test that passed because a 500 ms
blink never fired inside it may now see it fire once that test advances the
clock. That is the correct trade — the old arrangement raced the deterministic
scheduler and was a latent flake source independent of the abort — but it is the
one thing here that can surprise someone writing a blink or toast timing test
later. It is stated again in `src/undying_thread_guard.rs`'s module docs, where
that person will be reading.

## Review feedback

*On spec 1 (#190):*

1. **`scripts/run-tests.sh` must not exit 0 on "all summaries ok, cargo status
   non-zero".** Done — that arm now prints `VERDICT: FAIL`, says it is a
   teardown abort rather than a test failure, names #190 and `cargo tree -i
   async-io` as the starting point, and says not to re-run until it goes green.
   Exit 1. The "fewer `test result:` lines than announced binaries" detector is
   kept as the reviewer asked.
2. **Say the abort was never reproduced.** Done — its own paragraph above, with
   the aarch64-macOS numbers and the named check (`cargo test --lib` six times
   on an aarch64 Mac).
3. **Do not reconcile with the piped-exit-status problem here.** Done. No
   `pipefail` recommendation anywhere; `scripts/run-tests.sh` has no pipelines
   because it must not read one's status, not as advice to anyone else.
   `docs/running-tests.md` closes by saying explicitly that it is *not* the
   answer to the piping problem, so the file cannot be mistaken for the place
   that was solved.
4. *(Carry, not change)* **Repeat the deterministic-clock behaviour change.**
   Done — its own paragraph above.
5. *(Heads-up)* **#180 already touches `Cargo.toml`, `src/lib.rs`, `README.md`.**
   Confirmed present in the base (`52f3d91`, via `3973146`) and extended rather
   than replaced: the `smol` line in `[dependencies]` became a comment saying
   why there is no async runtime, the new `#[cfg(test)] mod
   undying_thread_guard;` sits beside the existing `mod build_profile_guard;`
   with a doc comment in the same shape, and README gained a `## Testing`
   section before `## License`.

*On spec 2 (#198):*

1. **Delete the `cd` and the `Cargo.toml` guard.** Done. The supervisor stages
   the script at `<workdir>/.git/tasks-verify` and runs it with cwd already at
   the repository root, so `$0` is not in this repo and the old `dirname`
   arithmetic worked only by coincidence of depth — a live hazard that would
   have failed every gpuikit build with a message blaming the repo layout the
   day that path changed. The script takes the cwd it is given. The header says
   so, so it does not get "fixed" back.
2. **Shebang and exec bit are inert; POSIX matters more than argued.** Done.
   `#!/bin/sh` kept for humans running it by hand only; no claim anywhere that
   the mode bit is load-bearing (the file is still `100755`, which now costs
   nothing and proves nothing). Every line is POSIX and the header says `sh` is
   the only invocation path there is.
3. **Do not pre-empt the cold-build timeout; put the reason in the header.**
   Done — the header's closing paragraph explains that a first suite round that
   times out reports `TimedOut`, which is never green, so the pull request still
   opens and routes to a human; only a *red* suite fails a build. So the cost of
   a cold run is one unverified pull request, once, and narrowing the command to
   dodge it would trade real coverage for a one-time cost.
4. *(Not asked to change, confirmed anyway)* `cargo test --lib` over anything
   wider is unchanged, and the two reasons are in the header. The deferred Rust
   guard asserting the script's contents is **not** added here, per the
   reviewer: it cannot be compile-checked without the whole gpui tree, and an
   uncompilable test file would break the exact command being delivered.

## Directions from the orchestrator

- **#190 first, #198 second.** Done, in that order and in that order of commits
  — a gate over an intermittently aborting suite is worse than no gate.
- **514 passed / 0 failed / 0.19s on the base, exit clean.** Taken as given and
  not read as disproof; it is why the argument above is structural and why the
  "what was not demonstrated" paragraph exists.
- **Budget for the cold `gpui` compile.** Done — this clone had neither
  `target/` nor a populated `~/.cargo/registry`, so the fetch was started
  alongside the edits and the build was the long pole of the run, as predicted.
  Final state: `cargo test --lib` is **517 passed, 0 failed, exit 0** (514 on
  the base plus this change's three guard tests), `cargo fmt --check` clean.
  `scripts/run-tests.sh --lib` reports `VERDICT: PASS`, and both of its failure
  arms were exercised for real against a stub `cargo`: an all-green run exiting
  134 gives `VERDICT: FAIL … teardown abort`, exit 1, and two announced binaries
  with one summary gives the #180 shortfall message, exit 1. `.tasks/verify` was
  run the way the supervisor runs it — copied outside the worktree, `sh <path>`
  from the repository root — and exits 0.
- **`.tasks/verify` must not run clippy with `-D warnings`.** Clippy is left out
  of the script entirely, rather than run without `-D warnings`. Two reasons: on
  this trunk it is 30 warnings and 0 errors (21 `type_complexity` on the
  `Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>>` callback-field idiom,
  3 `too_many_arguments` in `input.rs`, 6 nits), so with `-D warnings` every
  build against this repository would fail before doing anything and the gate
  would have to be reverted — and *without* `-D warnings` it gates nothing while
  adding a second full compile of the crate to a script whose one job is to
  report a status. Thirty lines of advice no one reads is not worth the wall
  clock. The header says so, and says adding clippy is a one-line follow-up once
  trunk is clean.
- **This build runs ungated and reports `Undeclared`.** Understood and expected;
  stated in the #198 paragraph above.
- **`sh <path>`, cwd at the repository root, exec bit not consulted.** Matches
  review item 1 and 2 on spec 2; nothing relies on a shebang or a mode bit.
- **#203 (no CI at all — every branch reports `CLEAN` for having no checks).**
  Not touched, as instructed. It is why `.tasks/verify` is the only gate this
  repository has, which is the reason the script is written to be the thing that
  reports a status and nothing else.
