# g03.005 - Clip Processing Contract And Export Tranche

Date: 2026-03-12
Roadmap: `docs/roadmaps/g03/005-clip-rendering-fades-and-nondestructive-processing-depth.md`

## Summary

Opened the first meaningful `g03.005` tranche by turning runtime clip
processing from a thin fade-length-plus-gain placeholder into an explicit
contract surface.

The landed runtime contract now includes:

- typed fade envelopes with `Linear`, `EqualPower`, and `SmoothStep` shapes
- typed gain envelopes with `Hold` and `Linear` shapes
- ordered per-clip treatment stages:
  - `Warp`
  - `FadeIn`
  - `GainShape`
  - `FadeOut`
- clip-processing snapshot/export that preserves realized warp ratio and
  project-tempo provenance alongside treatment ordering

## Evidence

Implemented in:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`

Focused proofs:

- `runtime_reconciles_clip_processing_against_media_and_warp_readiness`
  - verifies typed fade/gain treatment ordering and invalid hold-shape
    rejection
- `runtime_clip_processing_exports_treatment_surface_with_warp_and_automation`
  - verifies clip-processing export alongside tempo-map, warp, and automation
    snapshots in compact, multiline, and JSON report surfaces

## Deferred Scope

This tranche stops at contract, validation, and export depth. It does not yet
apply the new fade and gain envelopes through one realized engine-owned render
path for offline export or freeze work. That remains the next `g03.005` batch.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.005` by turning the typed clip-treatment contract into one
runtime-owned realized render path with reusable fade and gain application
helpers.
