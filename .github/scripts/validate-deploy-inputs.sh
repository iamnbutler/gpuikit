#!/usr/bin/env bash
#
# Judge the version and tag `release-deploy.yml` was handed, before either is
# written anywhere.
#
# That workflow is the one that runs `cargo publish`, and it reaches the shell
# by three routes: a `workflow_call` from `release.yml`, a `workflow_dispatch`
# whose `version` and `tag` are free-form strings a human types, and a `push`
# of any tag beginning with `v` and three dotted numbers — a filter whose
# trailing `*` accepts `v1.2.3";id;"` and, since a ref may carry one, an
# embedded newline.
# Nothing judged any of it. The values now arrive as `env:` bindings and this
# script is what judges them, in the first step of the first job, before
# anything is written to `$GITHUB_OUTPUT`.
#
# `$GITHUB_OUTPUT` is the reason this cannot wait for the `publish` job: it is
# a file of `name=value` lines, so a version carrying a newline sets further
# step outputs of its own on the way past.
#
# THIS SCRIPT STATES NO GRAMMAR. The version is handed to
# `validate-custom-version.sh`, which is where the one statement of the semver
# grammar lives; a second copy here would be a second thing to keep in step.
# Two things are decided here rather than there, because they are this
# workflow's policy rather than the grammar:
#
#   * The empty value is rejected. `validate-custom-version.sh` accepts it,
#     correctly — an empty `custom_version` is what every `version_type`
#     dispatch of `release.yml` sends, and that workflow computes the release
#     from the bump type instead. This one has no such fallback: an empty
#     version here is a wiring mistake, and it would be published.
#
#   * The tag must be `v` followed by the version. It gets no grammar of its
#     own, because on no route in are the two independent: `push` derives one
#     from the other, `release.yml`'s bump step computes `tag=v$NEW_VERSION`,
#     and a dispatch naming them separately has no legitimate reason to
#     disagree. Comparing them is also the only thing that has ever compared
#     them — a dispatch of version `0.8.0` with tag `v0.7.0` used to check out
#     one and publish the other.
#
# Usage: validate-deploy-inputs.sh <version> <tag>
# Exit:  0 usable · 1 not usable · 2 usage or wiring error
#
# The two failing codes are distinct on purpose, and a 2 out of the script
# delegated to stays a 2: 1 says the value is wrong, 2 says this step was wired
# up wrong. They are different fixes and the log should not conflate them.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <version> <tag>" >&2
  exit 2
fi

VERSION="$1"
TAG="$2"

if [ -z "$VERSION" ]; then
  cat >&2 <<EOF
Error: no version to deploy.

This workflow publishes the version it is given; there is nothing to compute
one from. Dispatch it with both a version and its tag, e.g. 0.8.0 and v0.8.0.
Nothing has been published or written.
EOF
  exit 1
fi

# The grammar lives next door. Located relative to this file so that the pair
# moves together and a caller's working directory cannot change the answer.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAMMAR="$HERE/validate-custom-version.sh"

if [ ! -f "$GRAMMAR" ]; then
  echo "Error: $GRAMMAR is missing, and it is what states the version grammar." >&2
  exit 2
fi

# stdout only is discarded: its success line names `custom_version`, an input
# this workflow does not have. Its rejection message goes to stderr and is kept,
# because it is the one that names the grammar. `|| STATUS=$?` so that errexit
# does not kill this script before it can pass the code on unchanged.
STATUS=0
bash "$GRAMMAR" "$VERSION" >/dev/null || STATUS=$?

if [ "$STATUS" -ne 0 ]; then
  echo "(read 'custom_version' above as this workflow's version to deploy.)" >&2
  exit "$STATUS"
fi

if [ "$TAG" != "v$VERSION" ]; then
  cat >&2 <<EOF
Error: the tag and the version do not name the same release.

  version : $(printf '%q' "$VERSION")
  tag     : $(printf '%q' "$TAG")

The tag is the ref that gets checked out and released; the version is what
gets published from it. They must be the same release, so the tag is 'v'
followed by the version — v$VERSION here. Nothing has been published or
written.
EOF
  exit 1
fi

echo "Deploying $VERSION as $TAG."
