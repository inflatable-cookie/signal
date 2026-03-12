# g03.004 - Tempo Map And Warp Contract Closure

Date: 2026-03-12
Owner: core-product
Roadmap: `docs/roadmaps/g03/004-tempo-map-stretch-and-warp-execution-substrate.md`

## Summary

Closed `g03.004` by making runtime tempo intent explicit, resolving project
tempo through a typed tempo-map seam, and exporting warp realization through
the same observation path already used for metering and automation.

Implemented in this tranche:

- `signal-runtime` now owns a tempo-map projection contract with explicit
  non-overlapping segment semantics and `Hold` or `Linear` interpolation.
- runtime resolves project tempo from one of three explicit sources:
  `DefaultFallback`, `TransportProjection`, or `TempoMapSegment`.
- warp snapshots now preserve both the resolved project tempo and the source
  that produced it, including active tempo-map segment identity when present.
- tempo-map and warp state now flow through `RuntimeObservationReport`,
  `RuntimeSupervisorReport`, and supervisor JSON instead of living behind
  isolated getters.

## Evidence

Focused proofs landed in:

- `crates/signal-runtime/src/runtime.rs`
  - `tempo_map_projection_requires_bounded_non_overlapping_segments`
  - `runtime_reconciles_warp_clips_against_media_readiness_and_project_tempo`
  - `runtime_tempo_map_projection_drives_warp_ratio_and_export_reports`

Those checks prove:

- invalid tempo-map ownership is rejected before runtime state becomes
  ambiguous
- ready and degraded warp states still surface against media readiness and
  ratio support constraints
- tempo-map segments can override transport tempo while runtime still falls
  back cleanly to transport tempo outside mapped regions
- supervisor-facing reports expose enough warp and tempo provenance to debug
  timing behavior without host-local inference

## Deferred Scope

Still deferred on purpose:

- no algorithm portfolio beyond the current baseline `Repitch` and
  `ElastiqueDraft` mode vocabulary
- no product-facing warp editing workflows, anchor authoring tools, or richer
  lane semantics
- no sample-domain stretched audio render path yet; this milestone stops at
  runtime-owned tempo resolution, readiness, and realized-ratio truth

## Validation

Passed:

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Next Task

Execute `g03.005` by defining reusable fade, gain-shape, and ordered
clip-treatment semantics, then prove nondestructive clip processing against
warped timing and automation-aware cases.
