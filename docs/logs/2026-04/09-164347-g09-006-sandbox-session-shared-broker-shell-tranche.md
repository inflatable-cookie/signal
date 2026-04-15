# 2026-04-09 - g09.006 sandbox session shared broker shell tranche

## Summary

Continued the active strict `g09.006` lane from the ready batch card by
extracting the common broker session and transport orchestration shell out of
both host `sandbox_sessions.rs` files into the runtime-owned broker support
layer.

## Changes

- extended
  `~/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
  with shared broker attach, prepared-recording, attached execution summary,
  VST3 broker execution sequence, and teardown helpers
- reexported the new shared broker shell through
  `~/Dev/projects/signal/crates/signal-runtime/src/lib.rs`
- changed
  `~/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  so the local host now keeps only AU/VST3 preparation and local env assembly
  at the edge
- changed
  `~/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  so the server host now keeps only AU/LV2/VST3 preparation, server env
  assembly, and LV2-specific execution handling at the edge
- preserved the format-specific and host-specific seams explicitly instead of
  trying to flatten AU/VST3/LV2 preparation into one forced abstraction

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Outcome

The highest-leverage shared broker session shell is now runtime-owned. The host
copies no longer carry parallel attach-record-teardown orchestration, and the
duplicated VST3 broker execution sequence is gone too. What remains in
`sandbox_sessions.rs` is smaller and more clearly edge-specific:

- AU and VST3 preparation/failure mapping
- local versus server broker environment assembly
- LV2-specific preparation and execution depth on the server host

That makes `g09.006` ready for a narrower reassessment rather than another
large blind extraction.

## Next Task

Reassess the remaining duplication in `sandbox_sessions.rs`, then, if
`g09.006` still has another broad shared-support seam, consolidate the
duplicated AU and VST3 broker-preparation and fault-recording shell while
keeping local/server environment assembly and LV2-specific behavior explicit at
the edges.
