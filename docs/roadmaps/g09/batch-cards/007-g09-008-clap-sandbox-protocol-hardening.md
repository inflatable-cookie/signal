# 007 - g09.008 CLAP Sandbox Protocol Hardening

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.008
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/contracts/076-low-level-correctness-safety-and-protocol-hardening-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/008-low-level-correctness-safety-and-protocol-hardening.md
Auto-start next card: no

## Objective

Continue `g09.008` with the next clear substrate-hardening seam: remove the
remaining panic-oriented CLAP sandbox request handling from the lifecycle
prepare and teardown path so internal drift returns typed protocol failures
instead of `expect(...)`-driven process termination.

## Scope

- harden the CLAP sandbox lifecycle prepare/activate/deactivate/reset path in
  `crates/signal-plugin-clap/src/clap_sandbox_harness/`
- replace the remaining request-handling `expect(...)` assumptions that are
  supposed to be guarded by `require_*` checks with explicit typed failure
  envelopes
- preserve the current runtime and sandbox continuity semantics; do not widen
  into shared-memory ownership cleanup yet
- add focused CLAP harness tests for the newly explicit drift cases

## Steps

1. Replace `expect(...)`-based validated-instance and state-payload assumptions
   in the CLAP sandbox lifecycle handlers with typed failure construction.
2. Keep protocol-drift and unsupported-state behavior machine-readable through
   the existing failure envelope path instead of panicking.
3. Add focused harness tests that prove the prepare/activate/deactivate/reset
   path now fails explicitly under internal drift.
4. Rerun the focused CLAP validation surface for the hardened seam.

## Acceptance Criteria

- request handling in the targeted CLAP lifecycle path no longer relies on
  panic for expected drift cases
- protocol drift returns typed failure envelopes instead of terminating the
  sandbox process
- focused CLAP harness tests cover the newly explicit failure cases
- focused validation passes

## Evidence Required

- batch log for the next `g09.008` tranche
- validation actually run
- explicit note if any remaining panic-oriented paths are intentionally left for
  the later shared-memory or broader protocol batch

## Outcome

The targeted CLAP lifecycle path no longer relies on panic-oriented
`expect(...)` handling. The prepare, activate, deactivate, reset, and
create-instance handlers now use explicit harness helpers for "validated
instance" and "instance state payload" projection, and protocol drift now
returns typed sandbox failure envelopes instead of terminating the process.

Focused drift coverage now proves the explicit protocol-violation path for:

- activate requests with an unprepared epoch
- prepare-state projection loss
- reset-state projection loss

This closes the CLAP lifecycle seam cleanly enough to hand the next hardening
batch to shared-memory lifecycle ownership rather than staying in more CLAP
teardown micro-fixes.

## Validation Run

- `cargo test -p signal-plugin-clap`
- `cargo check -p signal-ipc`
- `effigy health`

Validation note:
- `cargo test -p signal-plugin-clap` still reports the same pre-existing unused
  import warnings in `crates/signal-plugin-clap/src/tests/block_processing.rs`
  and `crates/signal-plugin-clap/src/tests/lifecycle.rs`; this batch did not
  add new warning noise.

## Stop Conditions

- the batch starts redesigning the whole sandbox protocol instead of hardening
  the existing lifecycle path
- the change needs a broader IPC ownership decision that belongs to the later
  shared-memory batch
- the work drifts into non-CLAP plugin adapters or runtime-wide recovery logic

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/008-g09-008-shared-memory-lifecycle-hardening.md`.
