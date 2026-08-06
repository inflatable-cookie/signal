#!/usr/bin/env bash
# Proves the release commit is consumable as a git dependency: builds a
# throwaway consumer against it and asserts every signal crate resolves from a
# git source rather than a path. Catches missing crates, path leakage, and
# manifests that only resolve because sibling checkouts exist.
#
# Signal is consumed by relative path during co-development and by git tag
# otherwise (docs/reference/consuming-signal.md), and the path arrangement is
# exactly what hides a broken tag: every sibling checkout is present locally, so
# a manifest that only resolves because of them looks fine right up until a
# consumer without them tries.
set -euo pipefail

release_repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$release_repo_root"

release_tmp=$(mktemp -d)
trap 'rm -rf "$release_tmp"' EXIT

release_consumer_root="$release_tmp/consumer"
mkdir -p "$release_consumer_root/src"

if [[ -z $(git status --porcelain) ]]; then
  release_source_root="$release_repo_root"
  release_source_commit=$(git rev-parse HEAD)
  release_source_kind=commit
else
  # Snapshot into a throwaway repo so the gate is runnable mid-development.
  # A dirty tree cannot be pointed at by `rev`, and refusing outright would
  # make this a gate you can only run once everything is already committed --
  # which is after the point where a finding is cheap to act on.
  release_source_root="$release_tmp/signal-source"
  mkdir -p "$release_source_root"
  release_source_list="$release_tmp/source-files.txt"
  while IFS= read -r -d '' release_source_path; do
    if [[ -e "$release_source_path" || -L "$release_source_path" ]]; then
      printf '%s\0' "$release_source_path"
    fi
  done < <(git ls-files --cached --others --exclude-standard -z) \
    > "$release_source_list"

  tar --null -T "$release_source_list" -cf - |
    tar -xf - -C "$release_source_root"

  (
    cd "$release_source_root"
    git init -q
    git add -A
    GIT_AUTHOR_DATE=2000-01-01T00:00:00Z \
      GIT_COMMITTER_DATE=2000-01-01T00:00:00Z \
      git \
      -c user.name=Signal \
      -c user.email=source-gate@invalid \
      commit -q -m 'Source consumer verification snapshot'
    test -z "$(git status --porcelain)"
  )

  release_source_commit=$(git -C "$release_source_root" rev-parse HEAD)
  release_source_kind=snapshot
fi

release_source_url="file://$release_source_root"

# Every crate a consumer actually names in its manifest, as of 2026-08-06:
# finch, soundcheck, soundcheck-library and loophole. Derived from what is
# consumed rather than from what looks public, because the claim being made is
# "the tag is usable by these projects", not "these crates seem important".
# `signal-dsp` is included as the DSP domain's representative even though no
# consumer names it directly yet.
#
# Extend when a consumer picks up a new crate. The five workspace members not
# listed -- signal-analysis-embed, signal-dsp-resample, signal-dsp-stretch-
# evidence, signal-ipc, signal-plugin-sandbox -- are reached transitively by
# the ones that are, so they still have to resolve from git for this to pass.
release_probe_crates=(
  signal-primitives
  signal-dsp
  signal-dsp-spectral
  signal-dsp-stretch
  signal-analysis
  signal-analysis-character
  signal-analysis-loudness
  signal-analysis-rhythm
  signal-analysis-tonal
  signal-graph
  signal-render-plane
  signal-runtime
  signal-host-local
  signal-hardware
  signal-hardware-coremidi
  signal-hardware-cpal
  signal-plugin
  signal-plugin-inventory
  signal-plugin-bridge
  signal-plugin-au
  signal-plugin-clap
  signal-plugin-lv2
  signal-plugin-vst3
)

source release-baselines/rust-toolchains.env

{
  cat <<EOF
[package]
name = "signal-source-consumer"
version = "0.0.0"
edition = "2024"
publish = false
rust-version = "${SIGNAL_GENERAL_MSRV%.*}"

[dependencies]
EOF
  for release_probe_crate in "${release_probe_crates[@]}"; do
    printf '%s = { git = "%s", rev = "%s" }\n' \
      "$release_probe_crate" "$release_source_url" "$release_source_commit"
  done
} > "$release_consumer_root/Cargo.toml"

{
  printf 'fn main() {\n'
  for release_probe_crate in "${release_probe_crates[@]}"; do
    printf '    use %s as _;\n' "${release_probe_crate//-/_}"
  done
  printf '}\n'
} > "$release_consumer_root/src/main.rs"

cargo generate-lockfile --manifest-path "$release_consumer_root/Cargo.toml"
cargo check --manifest-path "$release_consumer_root/Cargo.toml" --locked

release_metadata="$release_tmp/metadata.json"
cargo metadata \
  --manifest-path "$release_consumer_root/Cargo.toml" \
  --format-version 1 \
  --locked \
  > "$release_metadata"
# Every signal package except the throwaway consumer itself must carry a git
# source at the exact commit.
#
# The reference scripts select `.source != null`, which *excludes* a
# path-resolved crate instead of failing on it -- the leak being hunted becomes
# invisible, and only the count catches it. Signal has 28 workspace crates
# against 23 probes, so five could leak to path and the count would still clear
# `>= expected`. Selecting by name and requiring a git source of every survivor
# removes the slack; the count stays as a guard against the probe list silently
# shrinking.
jq -e \
  --arg commit "$release_source_commit" \
  --argjson expected "${#release_probe_crates[@]}" '
  [
    .packages[] |
    select(.name | startswith("signal-")) |
    select(.name != "signal-source-consumer")
  ] as $packages |
  ($packages | length) >= $expected and
  all($packages[];
    (.source // "") | startswith("git+file://") and endswith("#" + $commit)
  )
' "$release_metadata" > /dev/null

printf 'external source consumer passed at exact %s %s\n' \
  "$release_source_kind" \
  "$release_source_commit"
