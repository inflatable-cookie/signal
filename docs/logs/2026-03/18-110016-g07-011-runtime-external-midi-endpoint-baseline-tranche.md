# 2026-03-18 - g07.011 runtime external MIDI endpoint baseline tranche

## Summary

Completed Batch 11.2 of `g07.011` by materializing the first runtime-owned
external MIDI endpoint graph, device identity, capability, and lifecycle
receipt family through shared runtime and stable host-edge report surfaces.

This tranche turns the Batch 11.1 contract into a real bounded baseline instead
of leaving external MIDI endpoint truth implicit in host-local device tables or
product-local browser logic.

## Key changes

- added the new typed external MIDI receipt family to `signal-runtime`,
  including:
  - discovery and graph state
  - device and endpoint descriptors
  - bounded capability summaries
  - route-state and lifecycle meaning
- threaded the new snapshot through `RuntimeObservationReport`,
  `RuntimeSupervisorReport`, and report rendering across compact, multiline, and
  JSON surfaces
- kept runtime capture explicit with `Unavailable` external MIDI state when no
  host context is present
- aligned both stable host edges to project the same runtime-owned `Empty`
  graph baseline instead of rebuilding MIDI endpoint meaning in host-local code
- rolled the roadmap, contract, and architecture references forward so Batch
  11.3 can focus on the public proof seam

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_external_midi_endpoint_graph_snapshot_distinguishes_unavailable_from_empty -- --nocapture`
- `cargo test -p signal-runtime runtime_observation_report_render_json_surfaces_external_midi_snapshot -- --nocapture`
- `cargo test -p signal-runtime runtime_observation_and_supervisor_reports_surface_external_midi_endpoint_baseline -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline -- --nocapture`
- `cargo test -p signal-runtime --test public_contract_boundary --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This tranche closes runtime baseline depth, not the public consumer proof seam.
There is still no machine-readable descriptor or acceptance lane for external
MIDI yet, and richer MIDI 2.0, MPE, or control-surface device depth remains
deferred beyond this milestone.

## Next Task

Continue `g07.011` with Batch 11.3 by adding focused downstream-style proof
that the widened external MIDI endpoint graph, device identity, capability, and
lifecycle receipts remain consumable through shared runtime, supervisor, and
stable host-edge surfaces without host-local MIDI device reconstruction.
