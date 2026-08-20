# Running the tests

```sh
cargo test --lib
```

That is the suite. `.tasks/verify` runs exactly this, so the command a human
types and the command an automated build runs are the same one.

## Why `--lib`

Not a shortcut, and not a reduction in coverage worth reclaiming:

- **`--all-features` re-enables the `examples` feature.** Every `[[example]]`
  requires that no-op feature precisely so ordinary commands do not link eight
  copies of gpui at once; cargo offers no way to hold a feature back from
  `--all-features`, so that flag links all eight and the kernel kills a linker
  (gpuikit#180). `examples/README.md` has the full story.
- **`--all-targets` does the same thing** for the same reason.
- **A bare `cargo test` runs the doctests**, and rustdoc links one binary per
  fenced block — around 71 of them under `src/`. Same OOM class.
- **There is no `tests/` directory.** `--lib` skips no integration tests. The
  only coverage it gives up is doctests.

The lib is where the guard tests live for this reason: `src/build_profile_guard.rs`,
`src/release_version_guard.rs`, `src/release_input_validation.rs` and
`src/undying_thread_guard.rs` are all reachable from the one command that works
on a constrained machine.

## Why a run's exit status is not the whole answer

```sh
scripts/run-tests.sh --lib
```

Use this when a machine, or an unattended run, is deciding pass/fail. It runs
the same `cargo test`, captures the output to a file rather than piping it, and
judges from the harness's own `test result:` lines. Exit 0 pass, 1 failure, 2
nothing ran.

It exists because this repository has been misreported in both directions:

- **A binary that dies without reporting.** An OOM-killed link or a signal can
  leave a run with fewer `test result:` summaries than the `Running` lines that
  announced them. The script counts them and fails on a shortfall. This is
  gpuikit#180's shape.
- **A green run with a non-zero exit.** Every test passes, every summary says
  `ok`, and the process still exits non-zero because a thread with no exit path
  aborted during teardown — gpuikit#190, an `async-io` thread created by
  `smol::Timer`. The script **fails** this case rather than shrugging it off.
  The dependency that created that thread has been removed (see
  `src/undying_thread_guard.rs`), so if this arm ever fires again it is news:
  something regressed, or a second such thread arrived. Do not re-run until it
  goes green; find the thread. `cargo tree -i async-io` is where to start.

`scripts/run-tests.sh` judges the run it performs itself. It is not an answer to
the separate problem of *piping* a build's output and reading the pipeline's
status instead of the command's — that belongs to whatever does the piping, and
adding a recommendation about it here would only make this file look like the
place it was solved.
