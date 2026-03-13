# 013 - Control-Surface Transport, Mapping, And Feedback Baseline

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.011, g07.012
Vision tags: `CONTROL`, `HARDWARE`, `MIDI`

## Problem

Chorus already expects Signal to handle low-level control-surface discovery and
feedback transport, but Signal still needs a reusable baseline for that work.

## Goals

- [ ] define a reusable control-surface transport and feedback baseline
- [ ] keep mapping-relevant runtime meaning explicit without absorbing product policy
- [ ] support practical device feedback and controller I/O on one substrate

## Non-Goals

- [ ] no product-specific mapping UI or workflow layer
- [ ] no full scripting or extension policy yet

## Execution Plan

### Batch 13.1 - Control-Surface Contract

- [ ] define device identity, transport, feedback, and capability meaning
- [ ] align the contract with external MIDI endpoint and event surfaces

### Batch 13.2 - Runtime Baseline

- [ ] implement the first credible control-surface transport and feedback path
- [ ] keep host-visible state and diagnostics aligned with the contract

### Batch 13.3 - Focused Proof

- [ ] add focused proofs for control-surface transport and feedback behavior

## Acceptance Criteria

- [ ] Signal has an explicit control-surface transport and feedback baseline
- [ ] later mapping and extensibility work can build on the same device substrate
- [ ] low-level device transport remains outside app-local glue

## Risks And Mitigations

- Risk: control-surface work becomes privileged hardware exception handling.
- Mitigation: freeze one reusable device and feedback contract first.

## Evidence Requirements

- [ ] log each meaningful control-surface tranche
- [ ] run focused control-surface validation
- [ ] record deferred control-surface breadth explicitly

## Next Task

Continue `g07.014` by binding advanced hardware extensibility to the same
runtime-owned device policy rather than special-case integration.

