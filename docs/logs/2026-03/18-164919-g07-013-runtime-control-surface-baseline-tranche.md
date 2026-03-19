# 2026-03-18 - g07.013 Batch 13.2 Runtime Control-Surface Baseline Tranche

## Summary

Materialized the first runtime-owned control-surface transport, mapping, and
feedback baseline across runtime, supervisor, and stable host-edge surfaces.

## Work completed

- added `RuntimeControlSurfaceSnapshot` and per-device control-surface
  descriptors to `signal-runtime`
- derived control-surface transport posture, mapping posture, feedback
  readiness, and widened-expression capability from the closed external MIDI
  endpoint graph instead of host-local controller policy
- threaded the same control-surface snapshot family through runtime observation,
  supervisor export, and both stable host-edge report paths
- added focused runtime and host-edge tests for the new baseline and aligned the
  shared host-report JSON wrapper with the runtime-owned projection

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_control_surface_snapshot_derives_from_external_midi_baselines -- --nocapture`
- `cargo test -p signal-runtime runtime_observation_report_render_json_surfaces_external_midi_snapshot -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_control_surface_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_control_surface_baseline -- --nocapture`

## Deferred

- public runtime, supervisor-tools, and stable host-edge consumer proof for the
  widened control-surface receipt family
- machine-readable control-surface boundary descriptor and repo-owned acceptance
  lane
- richer vendor-protocol, display, haptic, and scripting-safe extensibility
  depth

## Next task

Continue `g07.013` with Batch 13.3 by adding focused downstream-style proof
that the widened control-surface transport, mapping-posture,
feedback-readiness, and capability receipts remain consumable through shared
runtime, supervisor, and stable host-edge surfaces without host-local
controller-policy reconstruction.
