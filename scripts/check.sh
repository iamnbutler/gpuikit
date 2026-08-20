#!/usr/bin/env bash
#
# scripts/check.sh — the checks examples/README.md documents, in the order they
# are quickest to run, with a failure that says which check failed and why.
#
#   scripts/check.sh          fmt, test --lib, check --all-targets (+ --features editor)
#   scripts/check.sh --link   the above, then actually link every example
#   scripts/check.sh --help   this text
#
# Two things this does that running the commands by hand does not:
#
#   * `set -euo pipefail`, and each cargo status read from ${PIPESTATUS[0]}
#     rather than from the pipe. A pipeline reports its *last* command's status,
#     so `cargo build --all-targets | tee build.log` exits 0 while the build was
#     OOM-killed. That is not hypothetical here: it is how a killed link was
#     once reported as a green run.
#
#   * It recognises `signal 9`. Linking any example is a whole-program link of
#     gpui; cargo picks -j from the CPU count and knows nothing about memory, so
#     a small machine can run more linkers at once than it has RAM for and the
#     kernel kills one. The message is `ld terminated with signal 9 [Killed]`,
#     which names no crate and no symbol and so reads as a compile error. It is
#     not one, and the way past it is fewer jobs, not a code change:
#
#       CARGO_BUILD_JOBS=1 scripts/check.sh
#
# CARGO_BUILD_JOBS is honoured throughout (cargo reads it itself; nothing here
# overrides -j).

set -euo pipefail

link_examples=0
for arg in "$@"; do
  case "$arg" in
    --link) link_examples=1 ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^#\{1,2\} \{0,1\}//'
      exit 0
      ;;
    *)
      echo "check.sh: unknown argument '$arg' (try --help)" >&2
      exit 2
      ;;
  esac
done

cd "$(dirname "$0")/.."

log_dir="${TMPDIR:-/tmp}/gpuikit-check.$$"
mkdir -p "$log_dir"
trap 'rm -rf "$log_dir"' EXIT

failed=0

# Run one check, teeing its output, and take the status from the command rather
# than from tee. ${PIPESTATUS[0]} is the whole point of this function.
run_check() {
  local name="$1"
  shift
  local log="$log_dir/${name//[^a-zA-Z0-9]/_}.log"
  echo "==> $name"
  set +e
  "$@" 2>&1 | tee "$log"
  local status="${PIPESTATUS[0]}"
  set -e
  if [ "$status" -ne 0 ]; then
    failed=1
    echo
    echo "!!! FAILED ($status): $name" >&2
    if grep -q "signal: 9\|signal 9\|SIGKILL" "$log"; then
      cat >&2 <<'MSG'

    That is the kernel's OOM killer, not a compile error. Linking an example
    is a whole-program link of gpui and takes roughly 800 MB; cargo runs one
    linker per job and picks -j from the CPU count, so N cores need about
    N * 0.8 GB free at the link step. Nothing in your code is wrong.

    Retry with fewer jobs:

        CARGO_BUILD_JOBS=1 scripts/check.sh

MSG
    fi
  fi
  echo
}

run_check "cargo fmt --check" cargo fmt --check
run_check "cargo test --lib" cargo test --lib
run_check "cargo check --all-targets" cargo check --all-targets
run_check "cargo check --all-targets --features editor" \
  cargo check --all-targets --features editor

if [ "$link_examples" -eq 1 ]; then
  # The heaviest documented configuration: this links every example, which is
  # the step that was being OOM-killed.
  run_check "cargo build --all-targets --features editor" \
    cargo build --all-targets --features editor
fi

if [ "$failed" -ne 0 ]; then
  echo "check.sh: at least one check failed (see above)" >&2
  exit 1
fi

echo "check.sh: all checks passed"
