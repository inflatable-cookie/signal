# Roadmaps

Status: active
Updated: 2026-07-20

## Why this section matters now

Roadmaps turn the Signal library/runtime strategy into executable batches.

## Scope

Use this section for:

- active implementation milestones
- generation control
- backlog and deferred work

## Layout

- `gNN/batch-cards/` optional per-generation execution cards
- `g*/`: generation roadmaps and closure records
- `generation-index.md`: generation history and rollover notes
- `backlog/`: deferred work only
- `templates/`: roadmap authoring support

## Current posture

`g10` is active. Its per-generation front door is
`docs/roadmaps/g10/README.md`. Phase one (002-009) completed the audit
remediation work: production-path declick + hardening, simulated/narration
mass demolition, workspace consolidation, and CI cleanup. Phase two (010-020)
completed the engine build-out on the surviving seed: graph-shaped plans,
stable identity, parameter fast path + automation, DSP kit, RT observability,
WYSIWYG bounce, output-time honesty, recording capture, disk streaming,
transport regions, and runtime endgame.

Phase three (021-031) established first-party stretch evidence and contracts.
`g10.029` is historical after consolidation removed rejected research and
stopped the narrow-proof queue. `g10.030` completed the OfflineHighQuality
successor decision. Its first isolated end-to-end candidate failed
anti-replica admission and was deleted. Its event-sealed replacement then
failed structural feasibility before implementation because the frozen impulse
rule is always `15` samples early. A final non-phase-vocoder feasibility study
found no family with a source-backed path through every whole-renderer gate.
The OfflineHighQuality successor program is closed on the frozen competitive
baseline. `g10.031` now owns a separate creative-stretch
path centered on `8x`. Three isolated diffusive-owner candidates were rejected
and deleted; the final candidate stopped at coefficient proof before renderer
admission. Range-owner reassessment paused the automatic spectral router and
selected explicit cyclic expansion through `8x` as the narrower next promise.
Its complete `CyclicGrain` candidate passed structural admission but failed the
first synthetic pitch row and was deleted. Batch 31.12 selected a materially
different correlation-aligned waveform family for a complete cyclic brief.
Batch 31.13 froze that brief without changing DSP. Batch 31.14 isolated
implementation failed structural search reachability and was deleted. Batch
31.15 found no third complete cyclic path and closed explicit `Cyclic`.
Batch 31.16 then reopened docs-only research by explicit operator decision.
Pinned PaulXStretch, CDP, and Potenza whole-path study found that the preferred
PaulXStretch default uses magnitude-only frame renewal and crossfade rather
than the recurrence tested by Signal's rejected spectral briefs. One
materially different neutral `Dream` family, `RenewalSpectral`, is selected
for one complete renderer brief. Batch 31.17 froze its exact map, transform,
phase renewal, linked stereo, pairwise synthesis, bounds, and gates without
changing DSP. Batch 31.18 implemented it once. Compile-only and structural
admission passed, but the first crest row measured `8.263162 dB` growth against
the frozen `6 dB` ceiling. The candidate was deleted before later synthetic or
listening gates. No candidate DSP is admitted on `main`; Batch 31.19 docs-only
crest-ownership reassessment is ready.
Offline artifacts still need a streaming artifact writer/cache target for full
peak-memory closure. Mono callback-state DSP has a
no-allocation proof, linked stereo is implemented, ratio scheduling has
source-frame alignment proof, and dynamic source projection is tracked. Render
plane use still needs an explicit source-fill and underrun contract, now
tracked in `g10.028`.
Product workflow planning remains deferred in `g10.025` until a real consumer
needs the Signal-owned contract.
Assessment driving phase two:
`docs/research/2026-06-11-post-demolition-assessment.md`. Plugin hosting,
MIDI, higher-quality SRC, and PDC stay in
`backlog/post-g10-rebuild-on-demand.md` until their prerequisites land.

`g06`, `g07`, `g08`, and `g09` are complete. The earlier post-`g08` backlog
note remains in
`docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`.
`g01` established the Rust workspace, engine, host/device
path, and plugin/runtime baseline; `g02` completed the first reusable DSP and
deep-analysis expansion on top of that foundation; `g03` completed the next
engine-oriented runtime depth queue; `g04` completed the reusable contract,
scheduling, portability, conformance, and release-baseline queue; `g05`
completed the widened backend, host-edge, publication-packaging,
downstream-automation, and generation-closeout queue; `g06` then closed the
next deeper Signal-owned runway around runtime recovery, instrumentation,
plugin-format breadth, MIDI/event expansion, hardware and external-I/O depth,
media-service depth, and shared acceptance evidence that tangibly moved
Loophole forward; `g07` then closed the bounded feature-expansion queue around
routing, Linux breadth, control-surface substrate, and sample-domain transform
services; and `g08` then closed the live-ownership and workflow-depth queue.

The previously completed continuation runway was:

- shared streaming spectral and resampling substrate
- rhythm structure and tempo continuity depth
- tonal and harmonic analysis depth
- loudness and dynamics depth
- transient/timbral descriptor packs
- embedding and benchmark hardening

The newly completed continuation runway was:

- routed mixer graph, buses, and topology depth
- runtime metering, loudness, and diagnostics export
- automation playback and control-resolution depth
- tempo-map, warp, clip-processing, and render substrate
- plugin device-chain execution, latency compensation, and state recall
- offline render, freeze, and stem export pipeline
- profiling, soak harnesses, and runtime hardening

The latest completed continuation runway was:

- crate/public-contract maturity and schema-freeze baseline
- multicore scheduling and anticipative execution depth
- runtime work orchestration and deferred-service policy
- hardware backend portability and clock-domain boundary depth
- plugin backend breadth and host-neutral delegation contracts
- consumer conformance, export stability, and release packaging

The latest completed continuation runway was:

- backend-neutral plugin capability and adapter breadth baseline
- shared host convenience API and consumer-edge contracts
- publication-grade packaging manifests and release automation receipts
- downstream conformance soak and release-acceptance automation
- generation closeout and promotion gate

The latest completed continuation runway was:

- runtime interruption, resumability, and recovery truth
- profiling, causal diagnostics, and deferred-work orchestration
- VST3 and AU adapter breadth plus richer generic MIDI/event semantics
- hardware supervision, external I/O, monitoring, and loopback depth
- media indexing, waveform analysis, preview, and metadata services
- fault injection, long-session soak, and Loophole-facing runtime readiness

The newly completed continuation runway was:

- spatial, multichannel, sidechain, and complex plugin-I/O depth
- LV2 plus deeper Linux plugin and hardware backend breadth
- external MIDI, control-surface, and advanced hardware device substrate
- fuller sample-domain time-stretch and transform-service depth

The latest completed continuation runway was:

- live Linux audio backend ownership and session lifecycle depth
- deeper LV2, complex plugin-I/O, and backend-native protocol breadth
- immersive routing, room-policy, and richer device-protocol substrate
- preview-device, audition, and product-adjacent workflow services that remain
  runtime-owned

The deferred continuation scope after `g09` is:

- broader repeated-run and environment-specific acceptance depth beyond the
  bounded `g08` closeout gate
- stronger shared downstream workflow hardening only when it remains
  Signal-owned and machine-readable
- product-local controller, browser, immersive-console, certification, and
  downstream launch workflows

## Strict lane posture

Signal is not currently running an active strict lane.

- strict-lane reference:
  `docs/specs/001-g09-lane-first-strict-adoption.md`
- current ready card: none

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Rollover guardrail

Do not open `gNN+1` while the current generation still has live roadmap files or stale strict-lane debris in the active specs tree.

Before rollover:

- every roadmap in the closing generation must be explicitly closed, paused, superseded, or moved to backlog
- the roadmap front doors must agree that the old generation is no longer the live queue
- `docs/specs/` must be purged so only live or near-live planning artifacts remain in the active tree

## Next Task

Use `docs/roadmaps/g10/README.md` as the active generation front door. Run
`g10.031` Batch 31.19 only. Reassess neutral-`Dream` crest ownership at
architecture level or close the owner. Keep rejected candidates, the
transparent successor lane, `g10.028`, other creative owners, routing, product
exposure, and render-plane integration closed.
