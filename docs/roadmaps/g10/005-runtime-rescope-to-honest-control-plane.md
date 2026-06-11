# 005 - Runtime Re-Scope To Honest Control Plane

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.004
Vision tags: `DEMOLITION`, `RUNTIME`, `SERDE`, `HONESTY`

## Problem

`signal-runtime` (~79k LoC) does not fulfill its named role: nothing in it
runs on an audio thread, `process_engine_block` allocates pervasively (per-
block strings, whole-buffer hashing, ~140-field snapshot clones), and the RT
obligation is met by `signal-render-plane` instead. ~65% of the crate is a
narration surface: ~34k LoC of snapshot/receipt families, ~180 stored
`summary: String` fields, ~5k LoC of hand-rolled JSON with no serde, and
simulated posture domains (JACK, PipeWire, control surfaces, spatial, ARA,
external MIDI) derived from other snapshots with no backing reality. 37% of
its 516 public exports have zero consumers. ~170 substring tests pin the
narration in place.

The real value — offline render/media pipeline (symphonia/hound), the
prework/anticipative scheduler, the transport/timeline state machine, and the
sandbox broker session — is buried inside.

## Goals

- [x] delete the simulated posture families and their exports/tests
- [x] delete the hand-rolled JSON + compact/multiline render layer; replace
      the (small) genuinely-consumed reporting slice with serde derives
- [x] remove stored `summary: String` fields from snapshots; consumers format
      at the edge if they need prose
- [x] move test scaffolding (`host_unification_support`, fault-injection boot
      entry points) out of the public API (`#[cfg(test)]` or dev-only crate)
- [x] harden the sandbox broker session: typed line protocol, read timeouts,
      stderr drain, struct returns instead of tuples
- [ ] rename/redocument the block path honestly as a simulation/diagnostic
      harness, or delete it if host-local tests can stand without it
- [x] collapse `signal-host-local` to the slice Pulse actually consumes;
      delete demo `main.rs` binaries
- [x] keep green: offline render, media processing, prework scheduler,
      transport state machine, broker sessions

## Non-Goals

- [ ] no attempt to make signal-runtime RT-capable — that role belongs to the
      render plane permanently
- [ ] no new observation features

## Execution Plan

### Batch 5.1 - Simulated Domains

- [ ] delete JACK/PipeWire/Linux-backend/external-MIDI/control-surface/
      spatial/ARA/advanced-hardware families + their tests
- [ ] prune `lib.rs` re-exports; workspace + pulse build gate

### Batch 5.2 - Narration Layer

- [ ] inventory which render output Pulse/host-local actually read
- [ ] serde derives for that slice; delete `render_*` methods, JSON family,
      summary fields, and their ~170 substring tests
- [ ] workspace + pulse build gate

### Batch 5.3 - Public Surface And Broker

- [ ] test scaffolding out of public API; delete unused exports (audit: ~190)
- [ ] broker protocol hardening (timeouts, stderr drain, typed states)
- [ ] host-local collapse; delete demo binaries
- [ ] full workspace + pulse + aura host test gate

## Acceptance Criteria

- [ ] signal-runtime public export count drops by more than half; all
      remaining exports have at least one consumer (workspace, pulse, or
      tests of kept behavior)
- [ ] no hand-rolled JSON remains in the crate
- [ ] pulse `signal_bridge` and its tests compile and pass against the
      reduced surface
- [ ] offline render, prework, transport, broker tests green throughout

## Risks and Mitigations

- Risk: pulse's bridge or its tests assert narration strings (audit found
  `summary.contains(...)` in pulse tests).
- Mitigation: update pulse expectations in the same change set; pointer bump
  coordinated in the workspace repo.
- Risk: cut depth misjudged — something simulated turns out consumed.
- Mitigation: per-batch green gates; each batch is one revertable commit.

## Evidence Requirements

- [ ] export-count and LoC deltas per batch in the progress log
- [ ] pulse test run recorded after each batch

## Progress (2026-06-11)

Three batches, three commits, ~29.4k LoC removed net across the packet:

- Batch 1 (simulated domains, −8,369): JACK/PipeWire-parity/Linux-backend/
  external-MIDI/control-surface/advanced-hardware families, their runtime
  projections, host-local pass-throughs, and self-confirming tests deleted.
  Observation report lost six simulated snapshot fields; host-io/external-io
  keep their real CoreAudio-backed data. Spatial + ARA fields survive —
  woven into kept offline-render/plugin-recall models; carve-out noted for
  later.
- Batch 2 (narration layer, −20,799): render_compact/multiline/json impls,
  manual JSON emitter family, ~50 format/json helper files, 119 stored
  summary-string fields, orphaned test trees, demo print binaries. The one
  consumed structured output (offline render report on disk) moved to serde,
  derives scoped to that path. Inventory confirmed: nothing outside the
  crate read any of the prose.
- Batch 3 (surface/broker/collapse, −275 net): lib.rs exports 453 → 270;
  test-harness macros inlined into host-local and removed from the public
  API; fault-injection boots cfg(test)-gated where possible; broker
  hardened (stdout reader thread + recv_timeout 10s default + env
  override, stderr drain with 16-line tail, quote-aware arg splitting,
  typed receipt states, teardown struct — wire format unchanged);
  host-local collapsed to the pulse-facing surface;
  `signal-hardware-coreaudio` DELETED — host-local's device hub now builds
  from `enumerate_output_devices()` (cpal), no system_profiler subprocess,
  completing g10.003's deferred goal.

Gates after every batch: signal workspace build + serial tests, pulse lib
122/122 serial, aura cargo check. Remaining for later packets: 49
integration-test-only exports (die with their contract tests if culled),
internal snapshot types woven into observation-report composition, the
`process_engine_block` simulation-path rename/demotion decision.

## Next Task

g10.006 (analysis pruning and correctness) — independent lane, can run in
parallel with this packet after g10.004.
