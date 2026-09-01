#!/usr/bin/env bash
# Focused proof for the option-2 prebuilt broker contract:
# 1) missing SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND fails with an actionable message
# 2) `effigy broker:provision` / this script's provisioner yields a usable absolute
#    executable that answers broker wire startup without consumer-side Cargo
set -euo pipefail

prove_repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$prove_repo_root"

printf '== missing-env diagnostic ==\n'
cargo test -p signal-runtime \
  sandbox_broker_support::tests::spawn_from_env_reports_actionable_missing_command \
  -- --exact --nocapture

printf '== provision absolute path ==\n'
prove_broker_command=$(bash "$prove_repo_root/scripts/provision-sandbox-broker.sh")
if [[ "$prove_broker_command" != /* ]]; then
  printf 'provisioner must print an absolute path, got: %s\n' "$prove_broker_command" >&2
  exit 1
fi
if [[ ! -x "$prove_broker_command" ]]; then
  printf 'provisioned path is not executable: %s\n' "$prove_broker_command" >&2
  exit 1
fi
printf 'provisioned: %s\n' "$prove_broker_command"

printf '== broker accepts prebuilt command (no cargo in argv) ==\n'
prove_output=$(printf 'status\nshutdown\n' | "$prove_broker_command")
printf '%s\n' "$prove_output"
printf '%s\n' "$prove_output" | rg -q 'state=starting'
printf '%s\n' "$prove_output" | rg -q 'state=ready'
printf '%s\n' "$prove_output" | rg -q 'status'
printf '%s\n' "$prove_output" | rg -q 'state=shutdown'

printf 'prebuilt broker contract proof ok\n'
