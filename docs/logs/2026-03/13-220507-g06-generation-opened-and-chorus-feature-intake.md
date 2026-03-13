# 2026-03-13 22:05:07 GMT - g06 generation opened and Chorus feature intake

## Summary

Opened `g06` as the next active Signal generation with a 20-milestone runway
that deliberately mixes runtime hardening and real missing feature breadth.
This opening also folded a focused Chorus sweep into Signal planning so
Loophole-facing feature demand is now mapped to Signal-owned milestones rather
than left as ad hoc planning notes.

## Work completed

- opened `docs/roadmaps/g06/README.md` as the active generation spine
- updated `docs/roadmaps/README.md`, `docs/roadmaps/generation-index.md`, and
  the promoted post-`g05` backlog item to reflect the new active generation
- seeded `g06.001` through `g06.020` as a full next-suite runway covering:
  - runtime interruption, recording/plugin/render recovery, and fault receipts
  - profiling, critical-path instrumentation, and deferred-work policy
  - VST3 and AU adapter baselines plus cross-adapter capability parity
  - generic MIDI/note-expression/plugin-event expansion
  - portable preset/state interchange and recall depth
  - hardware supervision, clocking, external-I/O, monitoring, and loopback depth
  - media indexing, waveform, preview, and analysis metadata services
  - fault-injection, long-session soak, and Loophole-readiness closeout

## Chorus feature sweep intake

The planning sweep pulled in Loophole-facing reusable feature needs that fit
Signal ownership:

- bounded AU/CLAP/VST3 plugin support from Chorus vision and plugin/routing
  architecture
- MIDI/event-model depth implied by editor foundations, MIDI recording, and
  performance-system architecture
- stronger hardware supervision, monitoring, external I/O, and measurement
  support from planned `chorus g03.017`
- waveform analysis, preview, and asset-analysis services from planned
  `chorus g03.018`
- integrated acceptance, fault injection, and long-session soak support from
  planned `chorus g03.019`

The same sweep also left several Chorus fronts intentionally out of `g06`
because they are not primarily Signal-owned reusable runtime work:

- plugin browser and UI-window workflows
- product-local editor and arrangement behavior
- remote/distributed profile orchestration and collaboration semantics
- AI/composer product workflows beyond reusable media-analysis inputs

## Why this matters

The old post-`g05` backlog item was too narrow for what Loophole actually needs
now. Signal had finished publication and boundary work, but Loophole still
needed both deeper runtime truth and real missing functionality such as AU/VST3,
MIDI/event expansion, external-I/O depth, and media services.

Opening `g06` this way means the separate Signal thread can keep moving for a
long time without another interim planning stop, while still staying inside
Signal-owned reusable boundaries.

## Validation

- `effigy qa:docs`
- `git diff --check`

## Next task

Start `g06.001` and freeze the runtime interruption and resumability contract
first, then move forward in dependency order through recovery depth,
instrumentation, plugin-format breadth, hardware/media services, and acceptance
work.
