#!/bin/sh
# The showcase, built for the browser. One statement of the recipe, used by
# a developer at a terminal and by `.github/workflows/pages.yml` alike:
#
#   scripts/showcase-web.sh serve                  # http://127.0.0.1:8081
#   scripts/showcase-web.sh build --release ...    # examples/showcase-web/dist/
#
# Everything after the subcommand is passed to trunk. Run from the repository
# root; trunk reads examples/showcase-web/Trunk.toml, which names the example
# and its feature, and index.html there points back at Cargo.toml.
#
# Why this is a script and not a line in the README: the web build needs a
# nightly (std is rebuilt with wasm atomics, which is `build-std`, unstable
# cargo), and `build-std` cannot be scoped to one target in `.cargo/config.toml`
# — a `[unstable]` table there would rebuild std for native nightly builds as
# well. So the two settings that must only ever apply to this build are set
# here, as environment, for this one process.
#
# The nightly is pinned to the date ci.yml's wasm job verifies against; bump
# the two together. Needs `rust-src` and the wasm32 target on that toolchain:
#
#   rustup toolchain install nightly-2026-08-30 --component rust-src --target wasm32-unknown-unknown
#
# POSIX sh, and `exec`, so trunk's exit status is this script's.

set -eu

if [ "$#" -eq 0 ]; then
    echo "usage: scripts/showcase-web.sh <serve|build> [trunk args...]" >&2
    exit 2
fi

RUSTUP_TOOLCHAIN=nightly-2026-08-30 \
CARGO_UNSTABLE_BUILD_STD=std,panic_abort \
exec trunk "$@" --config examples/showcase-web/Trunk.toml
