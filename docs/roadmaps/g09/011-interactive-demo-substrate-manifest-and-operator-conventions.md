# 011 - Interactive Demo Substrate, Manifest, And Operator Conventions

Status: draft
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `DEMOS`, `PROOF`, `OPERATIONS`
Contract refs: `079`

## Problem

Signal does not yet have a repo-owned demo substrate that maps crate claims to
interactive proof paths, launch commands, receipts, and known exclusions.

## Goals

- [ ] define one shared demo substrate and manifest format
- [ ] make demos launchable through repo-owned commands
- [ ] record crate-to-demo coverage explicitly

## Non-Goals

- [ ] no downstream app shell or polished product UI
- [ ] no replacement of normal unit or acceptance tests

## Execution Plan

### Batch 11.1 - Demo Program Shape

- [ ] define where demo binaries live and how they declare scenarios
- [ ] choose the machine-readable manifest schema for crate coverage, exclusions,
      commands, and expected manual checks
- [ ] decide which crates need dedicated demos versus domain-shared scenarios

### Batch 11.2 - Launch And Evidence Conventions

- [ ] add repo-owned launch tasks or commands for demo binaries
- [ ] define receipt/log capture conventions for demos
- [ ] document required operator notes, sample assets, and sunset rules for
      temporary fixtures

### Batch 11.3 - Coverage Matrix

- [ ] produce a crate-to-demo coverage matrix for the active workspace
- [ ] flag unsupported or deferred crates explicitly
- [ ] make the matrix part of the generation closeout evidence

## Acceptance Criteria

- [ ] every active crate maps to a demo or an explicit deferred status
- [ ] demos are runnable through repo-owned commands
- [ ] manifests and receipts make demo coverage inspectable

## Risks And Mitigations

- Risk: demos turn into ad hoc toy programs with no proof value.
- Mitigation: require manifest, coverage, and expected checks from the start.

- Risk: every crate gets its own needless binary.
- Mitigation: allow shared domain demos where that better matches real use.

## Evidence Requirements

- [ ] log the substrate and manifest decisions
- [ ] run `effigy qa:docs`
- [ ] run `effigy validate`
- [ ] record the first domain demo milestone unlocked

## Next Task

Continue with `g09.012` and build the first domain demos around runtime, host,
plugin, and hardware ownership.
