# g11.001 Batch 1.3 Render-Plane Consumer Wiring

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-83bcb179`
Branch: `t3code/host-assembly-wiring`

## Summary

Drove one offline render-plane plugin stage from a host-prepared
`RenderPluginProcessor`. The processor comes from
`prepare_plugin_processor`, not test-only construction.

## Path

scan compiled CLAP fixture → `prepare_plugin_processor(InProcess)` → Sum
stage `processor` handle → `render_plan_to_pcm`. Wet output is dry × fixture
gain (0.5).

## v1 vs deferred render-plane entry points

In v1:

- `prepare_plugin_processor` → `RenderPluginProcessor`
- Sum-stage `processor` on `RenderPlanSpec`
- `render_plan_to_pcm` offline bounce
- handle `process` / `process_with_events`

Deferred:

- live audio-thread host pumping
- Pulse workflow / host-owned plan compilation
- SharedSandbox backends
- trait-level `set_parameter_normalized` on in-process CLAP (inherent setter
  exists; the offline envelope seam is the trait default, which rejects)

## Validation

- `cargo test -p signal-host-local --test public_host_edge_plugin_processor`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/003-g11-001-host-edge-proof-and-closeout.md`.
