# 012 - Runtime, Host, Plugin, And Hardware Interactive Demo Suite

Status: active
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
- [x] add a sandbox lifecycle scenario that shows attach, ready, run, fault,
      and teardown behavior
- [ ] export manifests and receipts for these plugin scenarios

### Batch 12.2 - Runtime And Host Demo Paths

- [x] add a runtime execution and recovery demo that shows scheduler, recovery,
      and receipt surfaces live
- [x] add a local-versus-server host comparison scenario where shared recovery
      semantics can be inspected
- [ ] expose degraded cases or unsupported-path notes explicitly in manifests

Batch 12.2 Tranche 3 outcome:

- the existing `signal-host-local` and `signal-host-server` binaries now boot
  through an explicit supported VST3 demo bundle when no host demo override is
  already set
- this removes the immediate CLAP unsupported-path panic from the host demo
  bootstrap seam without claiming CLAP sandbox support exists
- host comparison output shaping is still deferred; the binaries are now
  bootstrapable but not yet promoted to a repo-owned host comparison demo

Batch 12.2 planning correction:

- the follow-on host comparison wrapper is not the next honest seam after all
- the remaining blocked work is the underlying CLAP host sandbox path still
  returning an explicit unsupported error in both local and server hosts
- `g09.012` therefore returns to that missing host-side CLAP behavior before
  the still-valid host comparison wrapper is promoted

Batch 12.2 Tranche 5 outcome:

- the real host-side CLAP sandbox path now compiles and runs in both
  `signal-host-local` and `signal-host-server`
- default host demo bring-up now carries explicit CLAP plugin ids, so
  `boot_default()` exercises the real CLAP path instead of failing on missing
  plugin identity
- the public cross-adapter parity proofs now assert bounded CLAP lifecycle
  truth rather than the old explicit unsupported-path receipt
- the next honest seam is again the deferred host comparison wrapper from card
  `025`

Batch 12.2 Tranche 6 outcome:

- the newly unblocked local and server host binaries are now wrapped by one
  repo-owned comparison manifest, launch task, operator notes file, and
  machine-readable receipt
- `signal-host-local` and `signal-host-server` are now live-covered in the demo
  coverage matrix
- the remaining `g09.012` plugin capability browsing and hardware topology
  seams still need fresh planning judgment rather than another automatic ready
  card

Batch 12.2 planning result after host comparison:

- plugin capability browsing is still not the next honest seam because the demo
  posture for owned scan roots and scan-result browsing remains underplanned
- hardware diagnostics is now the clearer bounded next batch because the host
  binaries already export native CoreAudio and simulated Linux backend/device
  truth through existing summary surfaces

### Batch 12.3 - Hardware Demo Paths

- [x] add a hardware topology and diagnostics scenario for simulated and native
      backends
- [ ] add a macOS-specific AU/CoreAudio scenario once `g09.004` lands
- [ ] add Linux-native backend and LV2 coverage once `g09.005` lands

Batch 12.3 Tranche 1 outcome:

- the existing `signal-host-local` and `signal-host-server` binaries are now
  wrapped by one repo-owned hardware diagnostics manifest, launch task,
  operator notes file, and machine-readable receipt
- `signal-hardware` and `signal-hardware-coreaudio` are now live-covered in the
  demo coverage matrix because the hardware diagnostics surface is real
- the hardware receipt keeps native CoreAudio and simulated Linux backend
  posture explicit and does not claim native Linux device ownership
- plugin capability browsing remains explicitly deferred after this tranche
  because demo-owned scan-root posture still wants fresh planning judgment

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

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, another bounded
host/runtime/hardware live-demo batch, or a continued planning pause.
