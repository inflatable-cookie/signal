# 011 - Interactive Demo Substrate, Manifest, And Operator Conventions

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `DEMOS`, `PROOF`, `OPERATIONS`
Contract refs: `079`

## Problem

Signal does not yet have a repo-owned demo substrate that maps crate claims to
interactive proof paths, launch commands, receipts, and known exclusions.

## Goals

- [x] define one shared demo substrate and manifest format
- [x] make demos launchable through repo-owned commands
- [x] record crate-to-demo coverage explicitly

## Non-Goals

- [ ] no downstream app shell or polished product UI
- [ ] no replacement of normal unit or acceptance tests

## Execution Plan

### Batch 11.1 - Demo Program Shape

- [x] define where demo binaries live and how they declare scenarios
- [x] choose the machine-readable manifest schema for crate coverage, exclusions,
      commands, and expected manual checks
- [x] decide which crates need dedicated demos versus domain-shared scenarios

### Batch 11.2 - Launch And Evidence Conventions

- [x] add repo-owned launch tasks or commands for demo binaries
- [x] define receipt/log capture conventions for demos
- [x] document required operator notes, sample assets, and sunset rules for
      temporary fixtures

### Batch 11.3 - Coverage Matrix

- [x] produce a crate-to-demo coverage matrix for the active workspace
- [x] flag unsupported or deferred crates explicitly
- [x] make the matrix part of the generation closeout evidence

## Acceptance Criteria

- [x] every active crate maps to a demo or an explicit deferred status
- [x] demos are runnable through repo-owned commands
- [x] manifests and receipts make demo coverage inspectable

## Risks And Mitigations

- Risk: demos turn into ad hoc toy programs with no proof value.
- Mitigation: require manifest, coverage, and expected checks from the start.

- Risk: every crate gets its own needless binary.
- Mitigation: allow shared domain demos where that better matches real use.

## Evidence Requirements

- [x] log the substrate and manifest decisions
- [x] run `effigy qa:docs`
- [ ] run `effigy validate`
- [x] record the first domain demo milestone unlocked

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md`.
