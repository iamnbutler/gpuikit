#!/usr/bin/env bash
#
# Refuse to release a version CHANGELOG.md does not name.
#
# `release.yml` computes the version to publish two ways that no longer agree.
# `custom_version` names it outright; `version_type` counts up FROM the version
# already in `Cargo.toml`, which since #170 is the version being *prepared*
# rather than the one already out. `version_type: minor` on a tree whose
# Cargo.toml says 0.8.0 therefore computes 0.9.0 and publishes it, skipping a
# version. `cargo publish` cannot be undone and a published version cannot be
# reused even after `cargo yank`, so until this script existed the only guard
# was a sentence in an input description.
#
# This repository has already lost a version to that: its tags are
# v0.3.0 v0.4.0 v0.5.0 v0.5.1 v0.7.0 — there is no v0.6.0, though CHANGELOG.md
# has a `## [0.6.0]` heading. The guard is against something with a precedent.
#
# The topmost `## [x.y.z]` heading is written by hand when a release is
# prepared and is already what a human checks, so comparing against it costs
# nothing. It also catches a release cut from a branch whose changelog was
# never updated, which is the same mistake by another route.
#
# Usage: verify-release-version.sh <version> [changelog-path]
# Exit:  0 they agree · 1 mismatch or no versioned heading · 2 usage error
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "usage: $0 <version> [changelog-path]" >&2
  exit 2
fi

VERSION="$1"
CHANGELOG="${2:-CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
  echo "Error: $CHANGELOG does not exist." >&2
  exit 2
fi

# The topmost versioned heading. The digit straight after the bracket is what
# keeps `## [Unreleased]` from being read as a version; the tail is `[^]]*`
# rather than numeric because release.yml's own custom_version regex accepts a
# `-beta.1` pre-release suffix, and `## [1.2.3-beta.1]` has to match.
#
# `|| true` because `set -e` plus a `grep` that matches nothing would kill the
# script before it could say why.
HEADING="$(grep -m1 -E '^## \[[0-9][^]]*\]' "$CHANGELOG" || true)"

if [ -z "$HEADING" ]; then
  echo "Error: $CHANGELOG has no '## [x.y.z]' heading at all." >&2
  echo "Add one for the version being released before dispatching a release." >&2
  exit 1
fi

TOP="$(printf '%s' "$HEADING" | sed -E 's/^## \[([^]]*)\].*/\1/')"

if [ "$TOP" != "$VERSION" ]; then
  cat >&2 <<EOF
Error: the version being released is not the one CHANGELOG.md names.

  computing to release : $VERSION
  topmost heading in $CHANGELOG : $TOP

Nothing has been written or tagged. Two ways out, and they are different fixes:

  1. $VERSION is the version you meant. Add '## [$VERSION]' to the top of
     $CHANGELOG (above '## [$TOP]'), with the notes for it.
  2. $TOP is the version you meant. Re-dispatch with custom_version: $TOP.
     This is almost always the answer when you dispatched with version_type,
     which counts UP from Cargo.toml and so skips the prepared version.
EOF
  exit 1
fi

echo "CHANGELOG.md names $VERSION as the version being released."
