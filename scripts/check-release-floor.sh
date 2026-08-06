#!/usr/bin/env bash
# Enforces the declared MSRV with real lints and tests, not a bare check.
#
# The distinction is load-bearing. Longhorn's floor violation appeared only
# under `--all-targets`: a dev-dependency pulled the feature that compiled the
# offending build script, and a bare `cargo check` passed and gave false
# confidence. Tests run too, because a crate can compile at the floor and still
# fail there.
#
# Signal has no per-crate MSRV override, so one pass suffices. If one is ever
# added, split this into an excluded-workspace pass plus a per-crate pass at the
# higher toolchain (see swallowtail's check-release-floor.sh).
set -euo pipefail

release_repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$release_repo_root"

source release-baselines/rust-toolchains.env

if ! rustup toolchain list | rg -q "^${SIGNAL_GENERAL_MSRV}(-|$)"; then
  printf 'missing required Rust toolchain: %s\n' "$SIGNAL_GENERAL_MSRV" >&2
  exit 1
fi

# Throttled like the other heavy gates in config/release.toml: two cores left
# free for the build, four for the test run, so the machine stays usable.
release_cores=$(sysctl -n hw.ncpu)

nice -n 5 rustup run "$SIGNAL_GENERAL_MSRV" cargo clippy \
  --workspace --all-targets --all-features --locked \
  --jobs $((release_cores - 2)) -- -D warnings
nice -n 5 rustup run "$SIGNAL_GENERAL_MSRV" cargo test \
  --workspace --locked \
  --jobs $((release_cores - 2)) -- --test-threads=$((release_cores - 4))

printf 'floor-toolchain Clippy and full tests passed at %s\n' \
  "$SIGNAL_GENERAL_MSRV"
