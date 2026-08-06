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

# Sync the lockfile to the workspace version before gating.
#
# `effigy release prepare` bumps `workspace.package.version` and *then* runs the
# gates, so at gate time Cargo.lock still records the previous version and
# `--locked` refuses outright -- the gate cannot run in the one flow it exists
# for. `cargo update -w` touches workspace members only; it is emphatically not
# `cargo generate-lockfile`, which re-resolves the whole graph and silently
# bumped ~40 third-party crates after the gates had run on the 0.1.0 prepare.
#
# Verified rather than trusted: every changed line must be a `version` line, and
# every added one must be the workspace version. A third-party bump would be a
# version line too, but not that version, so it is refused.
release_workspace_version=$(
  rg --max-count 1 '^version = "' Cargo.toml | sed 's/.*"\(.*\)"/\1/'
)
cargo update --workspace --quiet

release_lock_changes=$(
  git diff -- Cargo.lock | rg '^[-+]' | rg -v '^(\+\+\+|---)' || true
)
if [[ -n "$release_lock_changes" ]]; then
  release_lock_non_version=$(rg -v '^[-+]version = "' <<<"$release_lock_changes" || true)
  release_lock_foreign=$(
    rg '^\+version = ' <<<"$release_lock_changes" |
      rg -v "^\\+version = \"${release_workspace_version}\"\$" || true
  )
  if [[ -n "$release_lock_non_version" || -n "$release_lock_foreign" ]]; then
    printf 'cargo update -w moved more than the workspace version to %s:\n' \
      "$release_workspace_version" >&2
    printf '%s\n' "$release_lock_changes" >&2
    exit 1
  fi
  printf 'Cargo.lock synced to workspace version %s\n' \
    "$release_workspace_version"
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
