# 2026-03-18 - g07.012 runtime controller-expression receipts tranche

## Summary

Completed Batch 12.2 of `g07.012` by materializing the first runtime-owned
MIDI 2.0, MPE, and richer controller-expression receipt family across plugin
summary, runtime observation, and bounded external MIDI capability surfaces.

This tranche turns the Batch 12.1 contract into a real shared receipt baseline
instead of leaving wider controller-expression depth implicit in adapter-
private packet models or host-local capability guesses.

## Key changes

- widened `signal-plugin::EventPacketSummary` so note-expression evidence is
  split into pressure, timbre, and tuning families
- widened `signal-runtime::RuntimePluginEventSnapshot` with:
  - last-batch and aggregate richer note-expression family totals
  - typed `RuntimeControllerExpressionMpePosture`
  - typed `RuntimeControllerExpressionMidi2Posture`
- widened `RuntimeExternalMidiEndpointCapabilitySummary` so external MIDI
  capability surfaces can explicitly report guarded or unsupported richer
  controller-expression posture
- aligned runtime JSON and compact observation rendering to expose the widened
  controller-expression receipts through shared report surfaces
- strengthened focused runtime and host-edge proof fixtures so Batch 12.3 can
  close the consumer boundary on top of a real widened receipt family

## Validation

- `cargo fmt --all`
- `cargo test -p signal-plugin event_packet_summary_counts_richer_event_types -- --nocapture`
- `cargo test -p signal-runtime runtime_plugin_event_tracking_rolls_across_leases -- --nocapture`
- `cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_generic_event_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_generic_event_truth -- --nocapture`
- `cargo test -p signal-host-local --test public_host_edge_boundary --no-run`
- `cargo test -p signal-host-server --test public_host_edge_boundary --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Residual risk

This tranche closes runtime baseline depth, not the public consumer proof
seam. The widened controller-expression receipts are real now, but there is
still no dedicated machine-readable descriptor or acceptance lane for this
boundary, and richer MIDI 2.0 transport, negotiation, or live external-device
ownership remains deferred.

## Next Task

Continue `g07.012` with Batch 12.3 by adding focused downstream-style proof
that the widened MIDI 2.0, MPE, and richer controller-expression receipts
remain consumable through shared runtime, supervisor, and stable host-edge
surfaces without adapter-private packet reconstruction.
