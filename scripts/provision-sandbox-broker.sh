#!/usr/bin/env bash
# Explicit developer/CI provisioning for the sandbox broker executable.
#
# Builds `signal-plugin-sandbox` into an isolated target dir and prints the
# absolute executable path on stdout for:
#   export SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND="$(effigy broker:provision)"
#
# This step is separate from consumer startup. Consumer processes must never
# invoke Cargo to obtain the broker. The binary is host-local for the chosen
# profile; do not copy it across machines or OS/arch targets.
#
# Optional inputs (all explicit; defaults are local and reproducible):
#   SIGNAL_BROKER_TARGET_DIR  — Cargo --target-dir (default: <repo>/target/signal-broker)
#   SIGNAL_BROKER_PROFILE     — debug | release (default: debug)
set -euo pipefail

broker_repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$broker_repo_root"

broker_target_dir="${SIGNAL_BROKER_TARGET_DIR:-$broker_repo_root/target/signal-broker}"
broker_profile="${SIGNAL_BROKER_PROFILE:-debug}"

case "$broker_profile" in
  debug | release) ;;
  *)
    printf 'unsupported SIGNAL_BROKER_PROFILE=%s (expected debug or release)\n' \
      "$broker_profile" >&2
    exit 1
    ;;
esac

broker_profile_args=()
if [[ "$broker_profile" == "release" ]]; then
  broker_profile_args=(--release)
fi

printf 'provisioning signal-plugin-sandbox (%s) into %s\n' \
  "$broker_profile" "$broker_target_dir" >&2

if [[ ${#broker_profile_args[@]} -eq 0 ]]; then
  cargo build \
    -p signal-plugin-sandbox \
    --manifest-path "$broker_repo_root/Cargo.toml" \
    --target-dir "$broker_target_dir"
else
  cargo build \
    -p signal-plugin-sandbox \
    --manifest-path "$broker_repo_root/Cargo.toml" \
    --target-dir "$broker_target_dir" \
    "${broker_profile_args[@]}"
fi

broker_binary="$broker_target_dir/$broker_profile/signal-plugin-sandbox"
if [[ ! -x "$broker_binary" && -x "${broker_binary}.exe" ]]; then
  broker_binary="${broker_binary}.exe"
fi
if [[ ! -x "$broker_binary" ]]; then
  printf 'broker executable missing after build: %s\n' "$broker_binary" >&2
  exit 1
fi

broker_binary=$(cd "$(dirname "$broker_binary")" && pwd)/$(basename "$broker_binary")
printf '%s\n' "$broker_binary"
