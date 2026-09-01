#!/bin/sh
# Runs the test suite and decides pass/fail from what the harness *reported*,
# not from cargo's exit status alone.
#
# Two failure shapes have cost this repository real time, and they point in
# opposite directions:
#
#   * A test binary that dies without ever printing a summary — OOM-killed
#     while linking (#180), or aborted mid-run. Cargo can still exit 0 in some
#     pipelines, and a piped run reports the *pipeline's* status rather than
#     its own, so a killed run reads as a pass.
#   * A run in which every test passed and every summary says `ok`, but the
#     process still exited non-zero, because a thread with no exit path aborted
#     during teardown (#190).
#
# So this counts the summary lines against the number of test binaries cargo
# said it was running, and reports a disagreement between the summaries and the
# exit status as a failure in either direction. The second shape in particular
# is now *news*: #190 was fixed by deleting the `smol` dependency that created
# the `async-io` thread, so if all-green-with-a-non-zero-exit happens again,
# something regressed or a second such thread arrived. It must not be reported
# as success.
#
# Output is captured to a file rather than piped, deliberately: a pipeline
# hands back its last command's status, which is the very thing being guarded
# against.
#
# Usage: scripts/run-tests.sh [cargo test args…]   e.g. scripts/run-tests.sh --lib
#
# Exit: 0 pass · 1 failure · 2 nothing ran.

set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
log=$(mktemp "${TMPDIR:-/tmp}/gpuikit-tests.XXXXXX")
trap 'rm -f "$log"' EXIT INT TERM

echo "running: cargo test $*"
# The verdict below greps cargo's own announcements, so the run it judges is
# made in the one format those greps read: under CARGO_TERM_COLOR=always (CI
# sets it) every `Running` line arrives wrapped in escape codes and the count
# sees no binaries at all.
status=0
(cd "$root" && CARGO_TERM_COLOR=never cargo test "$@") >"$log" 2>&1 || status=$?
cat "$log"
echo

# `Running …` and `Doc-tests …` each announce one binary; `test result: …` is
# one summary. Every announced binary owes a summary.
binaries=$(grep -c -E '^[[:space:]]*(Running|Doc-tests) ' "$log" || true)
summaries=$(grep -c '^test result:' "$log" || true)
failed=$(grep -c '^test result: FAILED' "$log" || true)

echo "cargo exit: $status · test binaries: $binaries · summaries: $summaries"

if [ "$binaries" -eq 0 ]; then
    echo "VERDICT: NOTHING RAN — cargo announced no test binaries. Look above:"
    echo "  the build failed, or the target filter matched nothing."
    exit 2
fi

if [ "$summaries" -lt "$binaries" ]; then
    echo "VERDICT: FAIL — $binaries test binaries ran but only $summaries reported a"
    echo "  summary. One died without reporting: OOM kill, signal, or an abort"
    echo "  mid-run. This is gpuikit#180's shape; check for 'signal 9' above."
    exit 1
fi

if [ "$failed" -gt 0 ]; then
    echo "VERDICT: FAIL — $failed test binary/binaries reported failures."
    exit 1
fi

if [ "$status" -ne 0 ]; then
    echo "VERDICT: FAIL — every test passed and every summary says ok, yet cargo"
    echo "  exited $status. Nothing failed *during* the run, so this is a teardown"
    echo "  abort: some thread was still live when the process exited. That is"
    echo "  gpuikit#190's shape, and #190 was supposed to have removed the last"
    echo "  such thread by deleting the 'smol' dependency. Do not re-run until it"
    echo "  goes green — find the thread. 'cargo tree -i async-io' is the start."
    exit 1
fi

echo "VERDICT: PASS — $binaries test binaries, all reported ok, cargo exited 0."
