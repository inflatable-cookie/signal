# 2026-04-09 - g09.006 AU/VST3 preparation closeout

## Summary

Completed the active strict `g09.006` AU/VST3 preparation-and-fault batch, then
reassessed the remaining live duplication to decide whether another strict
ready card still exists inside the milestone.

## Changes

- extended
  `~/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
  with a shared prepared-session shell for AU/VST3 plus a shared
  protocol-violation prepare-fault recorder
- reexported the new shared support through
  `~/Dev/projects/signal/crates/signal-runtime/src/lib.rs`
- changed
  `~/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  so the local host now supplies only local environment assembly, instance-id
  prefixes, and format-specific edge behavior
- changed
  `~/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  so the server host now supplies only server environment assembly, server-only
  LV2 behavior, and the remaining format-specific edge behavior
- updated
  `~/Dev/projects/signal/docs/specs/batch-cards/003-g09-006-au-vst3-preparation-fault-shell.md`
  as complete
- updated the active currentness/front-door surfaces so the strict lane no
  longer claims there is a live ready card

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Reassessment Outcome

There is no longer a clearly broad, batch-cardable shared-support seam left in
`g09.006`. The remaining behavior in `sandbox_sessions.rs` is edge-specific:

- local versus server environment assembly and instance-id prefixes
- server-only LV2 preparation, negotiation, and execution depth
- smaller format-specific behavior that is no longer a broad shared shell

The strict lane should not invent another ready batch card from that residue.
It is now awaiting the next planning decision about whether `g09.006` closes
here or hands off into the next milestone.

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.006` closes here or hands off into the next milestone before creating a
new ready batch card.
