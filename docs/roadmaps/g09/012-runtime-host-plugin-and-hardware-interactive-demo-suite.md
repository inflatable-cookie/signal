# 012 - Runtime, Host, Plugin, And Hardware Interactive Demo Suite

Status: draft
Owner: core-product
Created: 2026-04-08
Depends on: g09.011
Vision tags: `DEMOS`, `RUNTIME`, `PLUGIN`, `HARDWARE`
Contract refs: `072`, `073`, `074`, `075`, `079`

## Problem

The most product-facing Signal claims live in runtime orchestration, plugin
hosting, host recovery, and hardware ownership, but there is no repo-owned
interactive suite that proves those claims live.

## Goals

- [ ] deliver interactive demos for runtime, host, plugin, and hardware seams
- [ ] cover both happy-path and degraded-path inspection where practical
- [ ] make these demos useful for both development verification and operator
      understanding

## Non-Goals

- [ ] no downstream DAW or browser UX shell
- [ ] no attempt to replace acceptance lanes with manual demos

## Execution Plan

### Batch 12.1 - Plugin And Sandbox Demo Paths

- [ ] add a plugin scan and capability browser scenario covering VST3, AU, and
      LV2 where supported
- [ ] add a sandbox lifecycle scenario that shows attach, ready, run, fault,
      and teardown behavior
- [ ] export manifests and receipts for these plugin scenarios

### Batch 12.2 - Runtime And Host Demo Paths

- [ ] add a runtime execution and recovery demo that shows scheduler, recovery,
      and receipt surfaces live
- [ ] add a local-versus-server host comparison scenario where shared recovery
      semantics can be inspected
- [ ] expose degraded cases or unsupported-path notes explicitly in manifests

### Batch 12.3 - Hardware Demo Paths

- [ ] add a hardware topology and diagnostics scenario for simulated and native
      backends
- [ ] add a macOS-specific AU/CoreAudio scenario once `g09.004` lands
- [ ] add Linux-native backend and LV2 coverage once `g09.005` lands

## Acceptance Criteria

- [ ] runtime, host, plugin, and hardware crates all map to live demo scenarios
- [ ] demo manifests identify supported versus deferred platform coverage
- [ ] operators can inspect both normal and degraded behavior for the main
      execution seams

## Risks And Mitigations

- Risk: demos overpromise platform coverage that is still deferred.
- Mitigation: require explicit supported/deferred manifest entries per scenario.

- Risk: demo scenarios become disconnected from actual runtime receipts.
- Mitigation: require direct reuse of runtime, host, and plugin receipt surfaces
  where possible.

## Evidence Requirements

- [ ] log each domain demo tranche
- [ ] run the demo launch tasks and record manifest output
- [ ] run `effigy health`
- [ ] record remaining deferred host/plugin/hardware demo coverage explicitly

## Next Task

Continue with `g09.013` and build the DSP, graph, and analysis demo suite plus
the generation's final audit-remediation proof posture.
