# 007 - Runtime Interface Decomposition And Test-Surface Normalization

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `RUNTIME`, `API`, `TESTS`
Contract refs: `003`, `009`, `015`, `075`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`

## Problem

`signal-runtime` no longer has one monolithic `interfaces.rs` wall, but it
still carries a few heavier internal assembly and test front doors that keep
contract evolution and review too implicit:

- the offline preview request assembly path is still a single heavy builder
- the runtime test front door still carries a broad import wall and shared slab

## Goals

- [ ] split runtime public interfaces into smaller semantic families
- [ ] move heavy assembly and preview builder logic behind internal seams
- [ ] normalize the runtime test tree around the same contract families

## Non-Goals

- [ ] no new runtime features in this milestone
- [ ] no downstream compatibility shims unless explicitly required

## Execution Plan

### Batch 7.1 - Public Family Map And First Live Seam

- [x] confirm that `interfaces.rs` is already a thin front door rather than the
      old oversized root
- [x] identify the remaining live runtime decomposition seam:
      `request_preview/request_assembly.rs`
- [x] identify the remaining test-normalization seam: `tests.rs`
- [x] freeze the first extraction boundary so execution can start without fresh
      planning decisions

### Batch 7.2 - Interface Extraction

- [x] carve the offline preview request assembly into explicit validation,
      resolution, policy, and summary helpers
- [x] keep the public DTO surface deliberate while moving the heavy assembly
      logic behind internal seams
- [x] preserve compile-only downstream-style imports through the runtime front
      door

### Batch 7.3 - Test-Tree Normalization

- [x] reduce the shared import wall in `tests.rs`
- [x] align test fixtures with the now-thinner public family map instead of one
      broad helper slab
- [x] split any remaining oversized runtime test roots only where they still
      resist the normalized family map

## Acceptance Criteria

- [x] remaining runtime roots are materially smaller and more domain-specific
- [x] public DTOs and request families are easier to version and review
- [x] test imports and fixtures follow the same domain boundaries

## Risks And Mitigations

- Risk: public reexports drift during extraction.
- Mitigation: treat public API diffs as a required review artifact for each
  tranche.

- Risk: test convenience drives public API shape again.
- Mitigation: keep fixture normalization as an explicit batch, not incidental
  cleanup.

## Evidence Requirements

- [x] log each runtime family extraction
- [x] run `cargo test -p signal-runtime --lib --no-run`
- [x] run `effigy health`
- [x] record any intentional breaking API moves explicitly in the batch log

## Strict-Lane Reassessment Outcome

`g09.007` is the correct next strict milestone. The earlier roadmap prose
overstated the remaining size of `interfaces.rs`; the real live seam is now the
heavier internal assembly wall in
`interfaces_offline_contract_family/request_preview/request_assembly.rs`,
followed by the broad `tests.rs` import surface. That is a clean milestone
handoff from `g09.006`: structural runtime decomposition remains active, but
the next batch should target the real internal assembly bottleneck instead of
re-solving a front door that is already thin.

## Batch 7.2 Tranche 1 Outcome

The first live `g09.007` extraction is now complete. The offline preview
request assembly path is no longer one mixed wall; validation, stem-target
resolution, and freeze-artifact derivation now live in narrow internal helpers
while `request_assembly.rs` keeps the stable orchestration entrypoint. That
means the next meaningful seam is no longer another similarly heavy internal
builder beside it. The next real runtime normalization wall is `tests.rs`,
which still carries a broad import front door and the pre-existing unused-
import warning cluster.

## Batch 7.3 Tranche 1 Outcome

The runtime test front door is now normalized. `tests.rs` no longer carries the
direct shared import slab; that wall lives in `tests/support.rs`, while the
root test entrypoint is back to being a small front door with local sink
helpers and mounted test families. The pre-existing five-item unused-import
warning cluster still exists, but it now lives in the dedicated support surface
instead of the root front door.

## Reassessment Outcome

I do not see another honest broad `g09.007` seam after the offline-preview
carveout and the runtime test front-door normalization. The remaining warning
cluster is too narrow to justify another strict ready card on its own, and the
larger follow-on work now belongs to the next milestone decision rather than to
more runtime-decomposition churn inside `g09.007`.

## Next Task

`g09.007` is complete. Continue the active strict lane from
`docs/specs/batch-cards/006-g09-008-graph-and-primitive-invariants.md`.
