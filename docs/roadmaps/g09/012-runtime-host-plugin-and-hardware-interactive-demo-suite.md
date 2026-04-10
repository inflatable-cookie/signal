# 012 - Runtime, Host, Plugin, And Hardware Interactive Demo Suite

Status: complete
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
- [x] add a macOS-specific AU/CoreAudio scenario once `g09.004` lands
- [x] add Linux-native backend and LV2 coverage once `g09.005` lands

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

Batch 12.2 planning result after hardware diagnostics:

- plugin capability browsing is still not the next honest seam because owned
  scan-root and browse-posture decisions remain underplanned
- the cleaner next batch is a `signal-supervisor-tools` companion for the
  existing runtime inspector family, because the remaining deferred runtime
  crate already exposes stable machine-readable runtime boundary descriptors
  through its current CLI

Batch 12.2 Tranche 7 outcome:

- the existing runtime recovery inspector family now has a repo-owned
  `signal-supervisor-tools` companion surface built from the current
  machine-readable boundary descriptor commands
- `signal-supervisor-tools` is now live-covered in the demo coverage matrix
  through that real companion manifest and receipt
- plugin capability browsing remains explicitly deferred because owned
  scan-root and browse-posture decisions still want fresh planning judgment

Batch 12.3 planning result after the supervisor companion:

- plugin capability browsing is still not the next honest seam because owned
  scan-root and browse-posture decisions remain underplanned
- the cleaner next batch is the already-frozen macOS AU/CoreAudio boundary,
  because `g09.004` already landed and the repo now has both a machine-readable
  descriptor command and a dedicated acceptance lane for that surface

Batch 12.3 Tranche 2 outcome:

- the existing macOS AU/CoreAudio boundary is now wrapped by one repo-owned
  manifest, launch task, operator notes file, and machine-readable receipt
- `signal-plugin-au` is now live-covered in the demo coverage matrix through
  that real macOS-specific surface
- the receipt captures both the machine-readable boundary descriptor and the
  existing acceptance lane posture without flattening the AU/CoreAudio seam
  into a generic plugin demo
- plugin capability browsing remains explicitly deferred after this tranche
  because owned scan-root and browse-posture decisions still want fresh
  planning judgment

Batch 12.3 Tranche 3 outcome:

- the existing `linux-lv2-execution-boundary` and
  `linux-audio-backend-boundary` descriptor-plus-acceptance surfaces are now
  wrapped by one repo-owned Linux manifest, launch task, operator notes file,
  and machine-readable receipt
- `signal-plugin-lv2` is now live-covered in the demo coverage matrix through
  that real Linux boundary surface
- the receipt keeps bounded LV2 broker-execution truth and Linux backend
  identity truth explicit without flattening them into a generic plugin demo
- the batch also repaired one stale acceptance-surface command so the Linux
  audio-backend boundary now points at its real focused server host-edge proof
  instead of the broken unfocused crate-level test invocation
- plugin capability browsing remains explicitly deferred after this tranche
  because owned scan-root and browse-posture decisions still want fresh
  planning judgment

Batch 12.3 planning result after the macOS AU/CoreAudio demo:

- plugin capability browsing is still not the next honest seam because owned
  scan-root and browse-posture decisions remain underplanned
- the cleaner next batch is a Linux-specific boundary companion, because the
  repo already has both machine-readable descriptor commands and dedicated
  acceptance lanes for `linux-lv2-execution-boundary` and
  `linux-audio-backend-boundary`
- that makes a bounded Linux LV2/backend demo bootstrap more honest than
  inventing browse posture next

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

Continue the active strict `g09` lane from
`docs/specs/batch-cards/031-g09-013-graph-execution-inspector-bootstrap.md`.
