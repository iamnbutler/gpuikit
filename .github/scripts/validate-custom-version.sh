#!/usr/bin/env bash
#
# Judge `release.yml`'s `custom_version` input before anything uses it.
#
# The step that used to do this interpolated `${{ inputs.custom_version }}`
# straight into its own `run:` body and then validated the result two lines
# below. `${{ }}` is substituted while the step's script is being *generated*,
# so by the time the check ran the value was already part of the script that
# was running it — a value carrying a quote and a newline put its second line
# outside the `if` and made the check unreachable. The value now reaches the
# shell only as an environment binding, and this script is what judges it, in
# a step of its own that runs first.
#
# The check is `[[ =~ ]]` rather than the `echo "$V" | grep -qE '^…$'` it
# replaces, and that is not a style preference: `grep` judges one line at a
# time, so `1.2.3` followed by a newline and anything at all matched on its
# first line and passed. The accepted value is written as `version=$NEW_VERSION`
# into `$GITHUB_OUTPUT`, where a second line sets further step outputs. Bash
# anchors `[[ =~ ]]` to the whole string.
#
# The grammar is deliberately the same one the old check named. Widening it
# here would desynchronise three things that are matched against the result:
# this script, `.github/scripts/verify-release-version.sh`'s comment about the
# `-beta.1` suffix, and `CHANGELOG.md`'s `## [x.y.z]` headings. If it ever
# changes, all three move together.
#
# An empty argument is what every `version_type` dispatch sends — the release
# comes from the bump type instead — so it is accepted, and says so.
#
# Usage: validate-custom-version.sh <version>
# Exit:  0 usable · 1 not a version · 2 usage error
#
# The two failing codes are distinct on purpose: 1 says the human who
# dispatched the release typed something wrong, 2 says this step was wired up
# wrong. They are different fixes and the log should not conflate them.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <version>" >&2
  echo "Pass the empty string when no custom version was named." >&2
  exit 2
fi

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "No custom_version given; the release will be computed from version_type."
  exit 0
fi

# Held in a variable, so the right-hand side of =~ must stay UNQUOTED — quoting
# it matches the pattern as a literal string and nothing ever passes.
SEMVER='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'

if [[ ! $VERSION =~ $SEMVER ]]; then
  cat >&2 <<EOF
Error: custom_version is not a version.

  given: $(printf '%q' "$VERSION")

Expected semver: 1.2.3, or 1.2.3-beta.1 with a pre-release suffix. No leading
'v', no surrounding whitespace, exactly three numeric components. Nothing has
been installed, written or tagged — re-dispatch with a corrected value.
EOF
  exit 1
fi

echo "custom_version is $VERSION."
