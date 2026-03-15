# g06.009 - Runtime VST3 Adapter Baseline Tranche

Date: 2026-03-15
Milestone: `g06.009`
Batch: `9.2`
Status: complete

## Summary

Landed the first real VST3 adapter baseline in Signal-owned crates. The repo
now has a Rust `signal-plugin-vst3` crate plus host integration that feeds VST3
discovery, sandbox lifecycle, and transport/session bring-up back into the
existing runtime-owned receipt family instead of leaving VST3 as package-map
intent or legacy-only reference code.

## What changed

- added `crates/signal-plugin-vst3`
- widened the workspace and host assemblies to depend on the new crate
- implemented bounded VST3 adapter surfaces for:
  - platform-specific scan roots, including explicit Linux VST3 roots
  - discovered plugin metadata with class/controller pairing
  - instance control surfaces
  - shared-memory session planning
- wired `signal-host-local` to:
  - scan VST3 roots on the local host path
  - publish VST3 discovered types through runtime-owned scan receipts
  - record VST3 sandbox lifecycle, instance-state, and transport attachment in
    runtime-owned lifecycle receipts
- wired `signal-host-server` to:
  - scan explicit Linux VST3 roots
  - publish Linux-hosted VST3 discovered types through runtime-owned receipts
  - record the same VST3 sandbox lifecycle and transport attachment path
- added focused proofs for:
  - VST3 adapter roots and session planning
  - local host VST3 discovery and lifecycle export
  - server host Linux-rooted VST3 discovery and lifecycle export

## Validation

- `cargo fmt --all`
- `cargo test -p signal-plugin-vst3 -- --nocapture`
- `cargo test -p signal-host-local local_host_vst3_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_vst3_scan_and_sandbox_surface_linux_runtime_owned_receipts -- --nocapture`

## Deferred

- public runtime/supervisor/stable-host-edge conformance proof for the new VST3
  path
- richer VST3 event, unit, program-list, and note-expression depth
- AU and broader cross-adapter parity work

## Next

Continue `g06.009` with Batch 9.3 by proving the new VST3 path remains
consumable through shared runtime, supervisor, and stable host-edge surfaces
without adapter-local reconstruction.
