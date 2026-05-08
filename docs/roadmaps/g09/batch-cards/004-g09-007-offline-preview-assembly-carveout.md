# 004 - g09.007 Offline Preview Assembly Carveout

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.007
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/contracts/015-offline-render-recovery-and-resumability-contract.md, docs/contracts/075-runtime-public-interface-decomposition-and-internal-assembly-boundary-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/007-runtime-interface-decomposition-and-test-surface-normalization.md
Auto-start next card: no

## Objective

Start `g09.007` with the first real remaining runtime decomposition seam:
carve the heavy offline preview request assembly out of
`request_preview/request_assembly.rs` into explicit internal helpers without
changing the deliberate public runtime front door.

## Scope

- decompose
  `crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/request_assembly.rs`
  into explicit internal helper responsibilities
- separate validation, target resolution, recall/freeze derivation, and final
  summary formatting where those responsibilities are currently mixed
- keep public DTOs and public runtime imports deliberate rather than leaking
  new internal helpers through broad reexports
- do not widen this batch into test-tree normalization yet

## Steps

1. Split the current offline preview request assembly wall into smaller
   internal helper units with narrow responsibilities.
2. Keep the public `RuntimeOfflineRenderContractPreview::from_runtime_state`
   entrypoint stable while pushing implementation detail behind internal seams.
3. Preserve compile-only downstream-style imports through the existing runtime
   front door.
4. Rerun the focused runtime validation surface for this extraction.

## Acceptance Criteria

- the remaining heavy offline preview assembly is materially smaller and easier
  to review by responsibility
- validation, resolution, policy derivation, and summary formatting are no
  longer fused into one wall
- the public runtime DTO/import surface remains deliberate
- focused runtime validation passes

## Evidence Required

- batch log for the next `g09.007` tranche
- validation actually run
- any intentional public import or breaking-surface movement stated explicitly

## Outcome

The offline preview request assembly wall is no longer one mixed builder.
`request_assembly.rs` is now the orchestration entrypoint, while validation,
stem-target resolution, and freeze-artifact derivation live in explicit
internal helpers under the same request-preview family. The public
`RuntimeOfflineRenderContractPreview::from_runtime_state` entrypoint stayed
stable, and no new internal helpers leaked through the runtime front door.

This leaves `g09.007` with a clearer next seam: the broad runtime test front
door in `tests.rs`, not another comparably large internal assembly wall right
next to this one.

## Validation Run

- `cargo test -p signal-runtime --lib --no-run`
- `effigy health`

Validation note:
- `cargo test -p signal-runtime --lib --no-run` still reports the pre-existing
  unused-import warning cluster in `crates/signal-runtime/src/tests.rs`; that
  warning belongs to the follow-on test-surface normalization seam rather than
  this batch.

## Stop Conditions

- the extraction starts leaking internal helpers through broad public reexports
- the batch turns into global test cleanup or another repo-wide normalization
  pass
- the public runtime boundary needs fresh design judgment not already captured
  in contract `075`

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/005-g09-007-runtime-tests-front-door-normalization.md`.
