# 005 - g09.007 Runtime Tests Front Door Normalization

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.007
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/contracts/015-offline-render-recovery-and-resumability-contract.md, docs/contracts/075-runtime-public-interface-decomposition-and-internal-assembly-boundary-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/007-runtime-interface-decomposition-and-test-surface-normalization.md
Auto-start next card: no

## Objective

Continue `g09.007` with the next real runtime decomposition seam: normalize the
runtime test front door in `crates/signal-runtime/src/tests.rs` so it stops
acting like one broad import wall and helper slab.

## Scope

- reduce the broad import wall in `crates/signal-runtime/src/tests.rs`
- align shared test imports and fixtures more closely to the thinner runtime
  family boundaries already in place
- clear the current pre-existing unused-import warning cluster if it falls out
  of the normalization cleanly
- do not widen this batch into unrelated test splitting or global warning
  cleanup outside the runtime test front door

## Steps

1. Move obviously over-broad imports in `tests.rs` down toward the narrower
   modules or helper surfaces that actually use them.
2. Reduce the shared front-door helper slab only where the usage boundary is
   already clear from the current test tree.
3. Keep the runtime test entrypoint deliberate rather than turning it into
   another incidental catch-all.
4. Rerun the focused runtime validation surface and confirm whether the current
   warning cluster disappears.

## Acceptance Criteria

- `tests.rs` is materially narrower and easier to review
- the shared import wall is reduced without breaking the existing runtime test
  family layout
- any removed warning noise is a byproduct of the structural cleanup, not
  standalone churn
- focused runtime validation passes

## Evidence Required

- batch log for the next `g09.007` tranche
- validation actually run
- explicit note if any warning cluster remains intentionally deferred

## Outcome

`tests.rs` is no longer the direct runtime-test import wall. The broad shared
imports now live in `tests/support.rs`, while `tests.rs` is back to being a
small front door with the local `TestSink`, fixture mount points, and one
shared support import. The existing runtime test family layout stayed intact:
top-level modules and deeper `super::*` / `super::super::*` chains still work
without re-splitting test domains.

The pre-existing unused-import warning cluster remains, but it is now attached
to the dedicated support surface instead of the root `tests.rs` front door. No
new warning family was introduced by the normalization batch.

## Validation Run

- `cargo test -p signal-runtime --lib --no-run`
- `effigy health`

Validation note:
- `cargo test -p signal-runtime --lib --no-run` still reports the same
  pre-existing five-item unused-import warning cluster, now in
  `crates/signal-runtime/src/tests/support.rs`:
  `BrokerInvalidationStage`, `LingeringCleanupMode`,
  `LingeringCleanupTrigger`, `RuntimeClipProcessingReadiness`, and
  `SandboxOperationFailureStage`.

## Stop Conditions

- the work starts turning into broad repo-wide warning cleanup
- the normalization requires fresh planning judgment about test architecture not
  already captured in contract `075`
- the batch starts re-splitting runtime test domains instead of normalizing the
  existing front door

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.007` closes here or hands off into `g09.008` before creating another
ready batch card.
