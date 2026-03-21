# 2026-03-19 17:57:01 - g08.004 runtime LV2 extension receipts tranche

## Summary

Landed the Batch 4.2 runtime-owned LV2 worker, URID, patch, and
extension-negotiation receipt baseline for `g08.004`.

## What changed

- added `RuntimeLv2ExtensionCapabilitySummary`,
  `RuntimeLv2ExtensionRecord`, and `RuntimeLv2ExtensionSnapshot` in
  `crates/signal-runtime/src/interfaces.rs`
- widened runtime observation and supervisor export so LV2 worker posture,
  URID negotiation posture, patch exchange posture, and extension-negotiation
  state now flow through one runtime-owned receipt family
- extended `signal-plugin-lv2` discovered type fixtures and host discovery
  conversion so LV2 extension capability evidence feeds the same runtime-owned
  seam instead of staying adapter-local
- aligned local and server stable host-edge proofs to export the same LV2
  extension snapshot without host-local reclassification
- updated the active roadmap, contract, architecture reference, and index
  surfaces for Batch 4.2 completion

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_linux_plugin_parity_coverage_tracks_policy_render_failure_and_restart_receipts -- --nocapture`
- `cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_lv2_extension_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_lv2_extension_truth -- --nocapture`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This tranche closes the first reusable LV2 extension receipt family, not the
full consumer-facing acceptance seam. Batch 4.3 still needs to prove the
widened LV2 extension boundary through shared runtime, supervisor, and stable
host-edge surfaces without introducing an LV2-only host policy model.

## Next Task

Continue `g08.004` with Batch 4.3 by proving the widened LV2 extension seam
through shared runtime, supervisor, and stable host-edge surfaces without
introducing an LV2-only host policy shell.
