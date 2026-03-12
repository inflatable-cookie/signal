# g03.005 - Clip Render Seam Closure

Date: 2026-03-12
Roadmap: `docs/roadmaps/g03/005-clip-rendering-fades-and-nondestructive-processing-depth.md`

## Summary

Closed `g03.005` by turning the clip-processing contract into one runtime-owned
realized render seam rather than leaving fade and gain application trapped in
report-only semantics.

The closed slice now includes:

- typed clip-render request/result DTOs
- a runtime-owned post-warp clip render helper
- timeline-relative fade and gain-envelope application
- clip-boundary silencing for out-of-range samples
- explicit rejection of pre-warp input for warp-enabled clip renders

## Evidence

Implemented in:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`

Focused proofs:

- `runtime_clip_render_path_applies_fade_gain_and_clip_bounds`
  - verifies exact rendered output for fade/gain application and clip-boundary
    silencing
- `runtime_clip_render_path_requires_post_warp_input_for_warp_enabled_clips`
  - verifies the bounded post-warp contract for warp-enabled clip rendering
- prior `g03.005` export proofs remain in place for report/export compatibility

## Deferred Scope

This closes clip rendering as a reusable seam, not the full offline export
pipeline. Multi-clip summing, freeze artifacts, and stem/export orchestration
remain deferred to `g03.007`. Plugin/device-chain execution and compensation
still need to land in `g03.006` before the offline path can reuse a complete
engine stack.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Execute `g03.006` by defining the first plugin/device-chain execution contract,
then prove latency-compensation and degraded-state/state-recall export through
runtime-owned surfaces.
