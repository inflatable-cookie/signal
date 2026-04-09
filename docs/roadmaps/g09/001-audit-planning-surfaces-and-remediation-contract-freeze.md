# 001 - Audit Planning Surfaces And Remediation Contract Freeze

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g08.020
Vision tags: `PLANNING`, `AUDIT`, `REMEDIATION`
Contract refs: `072`, `073`, `074`, `075`, `076`, `077`, `078`, `079`

## Problem

The post-`g08` audit identified concrete implementation gaps and structural
debt, but the repo did not yet have a planning generation, system inventory,
contract index, or remediation contracts that grouped those issues into one
coherent queue.

## Goals

- [ ] freeze the post-audit execution surface in architecture and contracts
- [ ] open one active generation for the remediation program
- [ ] turn every major audit issue into an executable roadmap dependency

## Non-Goals

- [ ] no implementation work beyond the planning surfaces themselves
- [ ] no speculative backlog beyond the audited issues and demo program

## Execution Plan

### Batch 1.1 - Inventory And Contract Front Doors

- [x] add and review `system-inventory.md` for the active Rust workspace
- [x] add and review `contract-index.md` so roadmap dependencies are explicit
- [x] define the missing remediation contracts for plugin realization, native
      backends, runtime decomposition, substrate hardening, fidelity uplift,
      rhythm resilience, and demos

### Batch 1.2 - Generation Open

- [x] open `g09` as the active generation in the roadmap index and section
      front doors
- [x] define the generation lanes, dependency order, and closure posture
- [x] keep the older post-`g08` backlog item visible rather than silently
      replacing it

### Batch 1.3 - Milestone Compilation

- [x] compile one roadmap file per remediation domain with explicit contract
      references
- [x] split plugin hosting and demo work into multiple milestones instead of
      one catch-all file
- [x] ensure each milestone carries executable batches, acceptance, evidence,
      and the next dependency

## Acceptance Criteria

- [x] the post-audit program has one active generation
- [x] every major audited issue is represented by a contract-backed roadmap
- [x] the repo has explicit planning front doors for inventory and contract
      lookup

## Risks And Mitigations

- Risk: roadmap work drifts ahead of missing contracts.
- Mitigation: freeze the remediation contracts before compiling the generation.

- Risk: the generation turns into a loose list of unrelated cleanup tasks.
- Mitigation: group the work into lanes that match shared implementation and
  proof seams.

## Evidence Requirements

- [x] record the planning-surface landing in repo logs
- [x] run `effigy qa:docs`
- [ ] run `effigy validate`
- [x] record the first execution milestone unlocked by the generation

## Batch 1 Outcome

`g09` is now open as the active post-audit remediation generation. The repo now
has the missing Northstar planning front doors in
`docs/architecture/system-inventory.md` and
`docs/contracts/contract-index.md`, eight new remediation contracts (`072`
through `079`), and a multi-lane roadmap set that breaks plugin hosting and
interactive demos into multiple executable milestones instead of catch-all
promises.

Focused docs validation passed through `effigy qa:docs` and `effigy qa:northstar`.
`effigy validate` is still blocked by pre-existing workspace issues outside this
planning batch:

- unresolved imports in `crates/signal-analysis-tonal/src/tests.rs`
- unused-import warnings in `crates/signal-plugin-clap/src/tests/*`

## Completion

`g09.001` is complete. The planning and contract baseline for the remediation
program is now in place.

## Next Task

Start `g09.002` and replace the fixture/demo plugin-hosting foundation with one
real shared discovery and sandbox execution substrate.
