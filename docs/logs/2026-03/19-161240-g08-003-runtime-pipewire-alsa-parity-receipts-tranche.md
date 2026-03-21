# 2026-03-19 16:12:40 - g08.003 runtime PipeWire/ALSA parity receipts tranche

## Summary

Landed the Batch 3.2 runtime-owned PipeWire/ALSA parity baseline for
`g08.003`.

## What changed

- added `RuntimePipeWireAlsaParitySnapshot` plus typed session-role,
  device-claim, stream-policy, and guarded-parity enums in
  `crates/signal-runtime/src/interfaces.rs`
- threaded the new parity receipt through runtime observation and supervisor
  export beside the existing Linux session and JACK coordination snapshots
- aligned local and server host report assembly to feed the same runtime-owned
  parity receipt into stable host-edge export
- added focused runtime and public host-edge proofs for:
  - direct ALSA callback parity
  - backend-managed PipeWire parity
  - recovery-guarded PipeWire parity
  - non-target local host parity
- updated the active roadmap, contract index, architecture reference, and
  generation pointers for Batch 3.2 completion

## Validation

- `cargo fmt --all`
- `effigy test --plan`
- `cargo test -p signal-runtime runtime_pipewire_alsa_parity_snapshot_derives_runtime_owned_parity_baselines -- --nocapture`
- `cargo test -p signal-runtime public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth -- --nocapture`

## Residual risk

This tranche closes the first reusable parity receipt family, not the full
consumer-facing acceptance seam. Batch 3.3 still needs to prove the widened
PipeWire/ALSA boundary through shared runtime, supervisor, and stable
host-edge surfaces without introducing a backend-specific descriptor that
duplicates later Linux acceptance work.

## Next Task

Continue `g08.003` with Batch 3.3 by proving the widened PipeWire and ALSA
parity seam through shared runtime, supervisor, and stable host-edge surfaces,
then decide whether a repo-owned acceptance descriptor belongs in this
milestone or the later Linux acceptance lane.
