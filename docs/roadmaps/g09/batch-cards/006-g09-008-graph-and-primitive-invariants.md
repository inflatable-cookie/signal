# 006 - g09.008 Graph And Primitive Invariants

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.008
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/001-shared-dsp-and-host-boundary.md, docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md, docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md, docs/contracts/076-low-level-correctness-safety-and-protocol-hardening-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/008-low-level-correctness-safety-and-protocol-hardening.md
Auto-start next card: no

## Objective

Start `g09.008` with the first honest substrate-hardening seam: make graph bus
layout adaptation and primitive audio-buffer construction reject or report
invalid states explicitly instead of silently normalizing them.

## Scope

- harden `crates/signal-graph/src/bus.rs` so unsupported channel-layout
  adaptation no longer silently returns a zeroed buffer as if it were a valid
  adaptation result
- harden `crates/signal-primitives/src/lib.rs` so invalid or lossy
  `AudioBuffer` construction is explicit rather than silently accepted
- add focused negative tests for the invalid graph and primitive cases touched
  by this seam
- keep the batch inside graph and primitive invariants; do not widen into CLAP
  protocol hardening or shared-memory lifecycle work yet

## Steps

1. Replace the current unsupported graph channel-layout adaptation fallback
   with an explicit failure or typed degraded-path result that callers can
   inspect.
2. Make `signal-primitives` reject zero-count or lossy interleaved-buffer
   states through an explicit constructor contract.
3. Update the affected graph call sites and tests so invalid adaptation and
   invalid primitive construction are machine-visible instead of implicit.
4. Rerun the focused graph and primitive validation surface for the hardened
   seam.

## Acceptance Criteria

- unsupported graph layout adaptation no longer silently returns a zeroed
  buffer as a valid adaptation outcome
- invalid or lossy interleaved audio-buffer construction is explicit
- focused negative tests exist for the newly explicit invalid cases
- focused validation passes

## Evidence Required

- batch log for the first `g09.008` tranche
- validation actually run
- explicit note if any caller-facing breaking invariant change is intentional

## Outcome

The first `g09.008` substrate-hardening seam is now complete. `signal-primitives`
has explicit fallible audio-buffer constructors for invalid channel layouts and
lossy interleaved sample counts, while the existing convenience constructors now
reject those invalid states instead of silently accepting them. The primitive
layer also now canonicalizes counted one- and two-channel layouts back to
`Mono` and `Stereo`, which closed the hidden mismatch where ordinary stereo
buffers could fall into the graph crate's unsupported-adaptation path.

On the graph side, unsupported channel-layout adaptation no longer looks like an
ordinary successful zeroed adaptation. The execution path now records explicit
degraded adaptation failures through the graph block report while preserving a
stable degraded output path, and focused graph tests now cover that boundary.

## Validation Run

- `cargo test -p signal-primitives`
- `cargo test -p signal-graph`
- `effigy health`

Breaking-change note:
- `signal-primitives` now rejects invalid zero-channel and lossy interleaved
  audio-buffer construction explicitly. The convenience constructors remain,
  but they now fail loudly instead of silently normalizing invalid input.

## Stop Conditions

- the change starts forcing a broad runtime rewrite instead of a bounded graph
  and primitive invariant hardening batch
- the graph hardening needs a larger receipt-design decision not already frozen
  in contract `076`
- the batch starts mixing in CLAP harness or shared-memory ownership work

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/007-g09-008-clap-sandbox-protocol-hardening.md`.
