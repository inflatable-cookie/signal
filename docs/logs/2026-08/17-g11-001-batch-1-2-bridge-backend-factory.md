# g11.001 Batch 1.2 Bridge Backend Factory

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-83bcb179`
Branch: `t3code/host-assembly-wiring`

## Summary

Landed `LocalRuntimeHost::prepare_plugin_processor`. The host now owns LV2
discovery beside CLAP/AU/VST3 and constructs bridge backends from scanned
types.

## Deliverables

- frozen factory on `LocalRuntimeHost`
- in-process construction for CLAP, VST3, AU, LV2
- DedicatedSandbox attach from a real broker audio lease (`load-plugin` →
  `activate` → `ShmPluginProcessor::attach`)
- SharedSandbox typed rejection (`shared_sandbox_unimplemented`)

## Fixture notes

- CLAP / VST3 / LV2 construction tests compile adapter fixtures with `rustc`
- AU construction uses a scanned load-key that resolves stock AUDelay through
  the system registry. AU has no compiled fixture compiler; the AudioComponent
  registrar cannot see temp bundles.
- DedicatedSandbox construction test enables the existing broker cargo-run
  plumbing; missing-lease rejection is covered without a broker child

## Validation

- `cargo test -p signal-host-local`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/002-g11-001-render-plane-consumer-wiring.md`.
