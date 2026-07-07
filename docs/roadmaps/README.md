# Roadmaps

Status: active
Updated: 2026-06-11

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

`g10` is active. Phase one complete (002-009): audit remediation —
production-path declick + hardening, ~98k LoC demolition, consolidation +
CI. Phase two planned (010-020): the engine build-out on the surviving seed
— graph-shaped plans and mixer realization (010), stable identity (011),
parameter fast path + automation (012), DSP kit (013), RT observability
(014), WYSIWYG bounce (015), output-time honesty (016), recording (017),
disk streaming (018), transport regions (019), runtime endgame (020).
Phase three planned (021-025): first-party high-quality stretch work —
real corpus/benchmark evidence (021), OfflineHighQuality DSP depth (022),
offline artifact scale and format depth (023), RealtimePreview tier (024),
and product workflow contract checkpoint (025).
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

Re-enter planning at the next-generation boundary before promoting another
strict execution lane.
