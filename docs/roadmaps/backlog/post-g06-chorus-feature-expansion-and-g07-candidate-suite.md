# Backlog: Post-g06 Chorus Feature Expansion And g07 Candidate Suite

Status: promoted
Priority: high
Estimated effort: full generation
Source: Chorus feature sweep on 2026-03-13

## Problem

`g06` now covers the highest-leverage runtime recovery, instrumentation,
plugin-format baseline, hardware supervision, media-service baseline, and soak
promotion work that Loophole needs immediately. That still leaves several
important Chorus-derived feature fronts either only implicit or clearly beyond
the current generation:

- spatial and multichannel execution depth
- sidechain, auxiliary, and multi-bus runtime behavior
- complex plugin I/O such as multi-output instruments and sidechain-capable FX
- Linux-native plugin breadth beyond CLAP/VST3, especially LV2
- Linux hardware backend depth beyond the current generic portability posture
- external MIDI, control-surface, and feedback-device substrate
- full sample-domain time-stretch depth beyond the current bounded warp
  realization contract

If these stay as scattered future ideas, Loophole will keep rediscovering them
feature by feature and Signal will need repeated interim planning passes.

## Proposed approach

When maintainers want the next major feature-breadth generation after `g06`,
promote this backlog item into a full `g07` suite. The recommended candidate
suite is:

### Lane A - Routing, Multichannel, And Spatial Execution

1. `g07.001` Canonical multichannel layout and channel-role substrate
2. `g07.002` Sidechain routing and secondary-input execution depth
3. `g07.003` Multi-bus graph execution and auxiliary topology depth
4. `g07.004` Plugin complex I/O topology and multi-output instrument depth
5. `g07.005` Spatial adapter execution baseline
6. `g07.006` Surround bed, object, and mix-policy expansion

### Lane B - Linux Runtime And Plugin Breadth

7. `g07.007` LV2 adapter baseline and Linux-native plugin lifecycle depth
8. `g07.008` Linux cross-adapter plugin parity and sandbox policy
9. `g07.009` Linux audio backend portability across ALSA, JACK, and PipeWire
10. `g07.010` Linux backend clocking, duplex, and endpoint-topology parity

### Lane C - MIDI Hardware, Control Surfaces, And Advanced Device Depth

11. `g07.011` External MIDI endpoint graph and device-identity baseline
12. `g07.012` MIDI 2.0, MPE, and richer controller-expression depth
13. `g07.013` Control-surface transport, mapping, and feedback baseline
14. `g07.014` Advanced hardware extensibility and scripting-safe device policy

### Lane D - Time-Stretch And Media Transformation Depth

15. `g07.015` Sample-domain time-stretch engine baseline
16. `g07.016` Warp-marker, transient-anchor, and tempo-assist analysis depth
17. `g07.017` Post-warp render, cache, and transform-artifact depth
18. `g07.018` Low-latency audition, scrub, and preview transform services

### Lane E - Acceptance And Promotion

19. `g07.019` Multichannel, Linux, time-stretch, and control-surface
    acceptance depth
20. `g07.020` Generation closeout and Loophole feature-readiness gate

## Why these candidates came out of Chorus

This suite is derived from repeated or explicit Chorus demand in:

- `chorus/specs/engine/spatial-adapters.md`
- `chorus/specs/engine/graph-snapshot-contract.md`
- `chorus/specs/engine/schedule-and-graph.md`
- `chorus/specs/signal-audio-backend-plugin-hosting.md`
- `chorus/specs/ipc/pulse/routing.md`
- `chorus/specs/ipc/pulse/control.md`
- `chorus/specs/ipc/pulse/hardware-io.md`
- `chorus/roadmaps/g03/017-hardware-integration-monitoring-and-external-io-depth.md`
- `chorus/roadmaps/g03/018-media-library-indexing-waveform-analysis-and-preview-services.md`
- `chorus/roadmaps/g05/014-control-surface-scripting-and-advanced-hardware-extensibility-depth.md`
- `chorus/vision/001-loophole-vision-blueprint-v1.md`
- `chorus/vision/002-loophole-refocus-matrix-v1.md`

## Scope boundary

These candidates belong in Signal because they widen reusable runtime or
adapter substrate. They do not imply that Signal should absorb:

- product-local browser or plugin-library UX
- plugin editor window-management policy
- Pulse authority semantics
- app-specific import/export workflows that are not reusable runtime substrate

## Promotion trigger

Promote this backlog item when at least one of the following becomes true:

- `g06` is sufficiently advanced that maintainers want the next full feature
  breadth generation ready without another planning pause
- Loophole pressure shifts from current recovery and baseline format breadth
  toward spatial, multichannel, Linux-native, control-surface, or deeper
  time-stretch capability
- maintainers want the next Signal thread to work against a future queue that
  is explicitly feature-forward rather than only hardening-forward

## Success criteria

- [x] the post-`g06` suite names concrete Signal-owned feature milestones rather
      than vague “more functionality” buckets
- [x] the suite is clearly grounded in Chorus-derived Loophole demand
- [x] the suite keeps ownership inside reusable runtime, adapter, hardware,
      media, and acceptance substrate instead of host-local UX
- [x] maintainers can open `g07` directly from this candidate suite without a
      fresh feature-discovery pass

## Risks

- spatial and multichannel depth can sprawl into product-specific mixing UX if
  the runtime contract is not frozen first
- Linux support can fragment across plugins and hardware unless adapter and
  backend policy remain explicit
- control-surface work can become privileged exception handling unless it stays
  within a reusable mapping and capability contract
- time-stretch can absorb too much algorithm experimentation unless the runtime
  execution and artifact surfaces stay bounded

## Next Task

PROMOTED into active `g07` on 2026-03-16 after `g06` closeout. Continue
`g07.001` by freezing the routing and multichannel substrate before widening
sidechain, spatial, Linux, control, or time-stretch implementation depth.
