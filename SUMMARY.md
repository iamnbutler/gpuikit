# Judge release-deploy.yml's outside values before anything uses them

`.github/workflows/release-deploy.yml` interpolated `${{ }}` into five `run:` bodies — a pushed
tag name, two free-form `workflow_dispatch` inputs, and the step outputs derived from them — in
the workflow that runs `cargo publish`, and nothing in the file validated any of it. `${{ }}` is
substituted while a step's script is being *generated*, so those values were code rather than
data, and the `push:` filter's trailing `*` accepts a tag named `v1.2.3";id;"`. All five bodies
now read their values as `env:` bindings, and a new first step in `prepare` hands the version and
tag to `.github/scripts/validate-deploy-inputs.sh` before either is written to `$GITHUB_OUTPUT` —
which is where validation has to happen rather than in `publish`, because `$GITHUB_OUTPUT` is a
file of `name=value` lines and an unjudged version carrying a newline sets further step outputs of
its own on the way past. `prepare` had no checkout at all; it has one now, deliberately without a
`ref:`, since the tag is the thing being judged. The new validator states no grammar of its own:
it delegates the version to `validate-custom-version.sh`, rejects the empty value (this workflow
has no bump type to compute a release from), and requires the tag to be `v` + the version rather
than giving it a second grammar. `CARGO_REGISTRY_TOKEN` moves from the workflow-level `env:` onto
the `publish` job, so `prepare` — the job that handles an unjudged tag name — no longer runs with
the crates.io credential in scope.

`src/release_input_validation.rs` now holds both workflows to the no-interpolation rule through a
`WORKFLOWS` table of `(name, yaml, run-body floor)`, and the "release-deploy.yml is deliberately
out of scope" paragraph is gone from all three places it appeared (the module doc, `release.yml`'s
header, and the `mod` declaration in `src/lib.rs`). Eleven new tests cover release-deploy: the
checkout precedes the validator, none of the six outside-value expressions reaches a `run:` body,
the validator precedes the `$GITHUB_OUTPUT` writes / `cargo publish` / the release action, the
step binds all four routes and carries no `if:`, the token sits inside the `publish` job's byte
range, the script delegates rather than restates the grammar, and accept/reject/usage/message
tests for the script itself. Each guard was mutation-tested: reintroducing an interpolation,
moving the token back to workflow level, deleting the checkout, deleting the validator call,
dropping a third workflow into the directory and copying a regex into the new script each fail
with a message naming the fix. The step was also simulated by hand under `bash -e` — a ref of
`v1.2.3\nversion=9.9.9` exits 1 with `$GITHUB_OUTPUT` empty, and an ordinary push and dispatch
pass through unchanged.

## Review feedback

- **Close the `WORKFLOWS` table rather than documenting it.** Done.
  `every_workflow_in_the_directory_is_accounted_for` reads `.github/workflows/` at test time (not
  `include_str!`, for the reason given) and fails if any `.yml`/`.yaml` file there is in neither
  `WORKFLOWS` nor `EXEMPT`; it also fails if a name in either list has vanished from disk.
  `EXEMPT: [(&str, &str); 0]` is the escape hatch — a named, empty list of `(file, reason)` pairs
  rather than an absence. Verified by dropping a `ci.yml` into the directory: the suite goes red
  with a message naming the file and both lists.
- **`without_comments()` applied to both ordering tests.** Kept, applied to release-deploy's new
  ordering test and to `release.yml`'s existing one.
- **The tag having no grammar of its own.** Kept: `[ "$TAG" = "v$VERSION" ]`, no regex, and the
  delegation test asserts the script contains neither `[0-9]` nor `=~`.
- **The `push:` filter staying as it is.** Unchanged, for the reason in the spec; the reject tests
  cover the exact ref names it lets through.
- **The token move onto the `publish` job.** Done, with a test.
- **This change stands alone.** Nothing here references the changelog-guard change: no hooks, no
  placeholders, no prose about it, and `release_version_guard.rs` is untouched. Validation lives in
  `prepare` with its own checkout.
- **`WORKFLOWS` entries are floors, not equalities.** `release-deploy.yml` has 9 `run:` bodies and
  a floor of 8, so the change that adds one stays green.

## Directions

- **Start the compile first, never two cargo invocations at once, report what I saw.** Followed: a
  `cargo build --lib` ran in the background while I read, and every later cargo call waited for the
  previous one to exit.
- **The intermittent `SIGABRT` in teardown (#190).** Not encountered; the final run exited 0 and I
  read the summary line rather than the exit code alone. Nothing was piped through `tee`/`tail` in
  a way that would mask a failure — the greps I used for mutation testing were diagnostic, and the
  verification run below reports the summary line directly.
- **Ships alone; validation in `prepare`; floor not equality.** All three as described above.
- No direction was declined, and nothing in the directions or the review conflicted with the spec.

Verification: PASSED — `cargo test --lib` (482 passed, 0 failed) and `cargo fmt -- --check` (clean)
