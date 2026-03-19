# 2026-03-17 - g07.008 Linux parity boundary closure and g07.009 handoff

## Summary

Closed Batch 8.3 of `g07.008` by proving the widened Linux plugin parity and
sandbox-policy receipt family through shared runtime, the stable server
host-edge surface, and a machine-readable supervisor-tools descriptor.

This closes `g07.008` on one bounded Linux plugin vocabulary for CLAP, VST3,
and LV2 and moves the active queue to `g07.009`.

## Key changes

- added a downstream-style public runtime proof for Linux-specific parity band,
  Linux support, preferred sandbox outcome, strict-sandbox default, restart,
  rebindability, and failure posture
- added a stable server host-edge proof that `RuntimeSupervisorReport` forwards
  the same Linux plugin vocabulary without host-local portability matrices
- added the machine-readable
  `signal.runtime.linux-plugin-parity-boundary` descriptor to
  `signal-supervisor-tools`
- added the repo-owned acceptance lane
  `effigy acceptance:linux-plugin-parity-boundary`
- closed the `g07.008` roadmap and contract trail and activated `g07.009`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_plugin_parity_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_linux_plugin_parity_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools linux_plugin_parity_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-linux-plugin-parity-boundary --format=json`
- `effigy acceptance:linux-plugin-parity-boundary --repo .`

## Residual risk

This closes the bounded Linux plugin parity seam, not Linux hardware backend
portability, richer CLAP or VST3 extension parity, or deeper LV2 worker, UI,
patch, or URID breadth. Those remain subsequent `g07` work.

## Next Task

Continue `g07.009` with Batch 9.1 by freezing the runtime-owned Linux audio
backend portability contract across ALSA, JACK, and PipeWire on top of the
now-closed Linux plugin parity and sandbox-policy boundary.
