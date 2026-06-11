# 005 - Runtime Re-Scope To Honest Control Plane

Status: planned
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

- [ ] delete the simulated posture families and their exports/tests
- [ ] delete the hand-rolled JSON + compact/multiline render layer; replace
      the (small) genuinely-consumed reporting slice with serde derives
- [ ] remove stored `summary: String` fields from snapshots; consumers format
      at the edge if they need prose
- [ ] move test scaffolding (`host_unification_support`, fault-injection boot
      entry points) out of the public API (`#[cfg(test)]` or dev-only crate)
- [ ] harden the sandbox broker session: typed line protocol, read timeouts,
      stderr drain, struct returns instead of tuples
- [ ] rename/redocument the block path honestly as a simulation/diagnostic
      harness, or delete it if host-local tests can stand without it
- [ ] collapse `signal-host-local` to the slice Pulse actually consumes;
      delete demo `main.rs` binaries
- [ ] keep green: offline render, media processing, prework scheduler,
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

## Next Task

g10.006 (analysis pruning and correctness) — independent lane, can run in
parallel with this packet after g10.004.
