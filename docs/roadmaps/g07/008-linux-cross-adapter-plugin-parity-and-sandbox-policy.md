# 008 - Linux Cross-Adapter Plugin Parity And Sandbox Policy

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.007, g06.003
Vision tags: `PLUGINS`, `LINUX`, `CONFORMANCE`

## Problem

Adding LV2 alone is not enough. Linux consumers still need one explicit view of
which plugin behaviors are portable, how sandbox policy applies, and where
unsupported-state receipts should appear.

## Goals

- [ ] define Linux cross-adapter plugin parity across CLAP, VST3, and LV2
- [ ] align Linux plugin breadth with the shared sandbox and placement-policy model
- [ ] keep runtime-owned portability and fallback behavior explicit

## Non-Goals

- [ ] no marketing feature matrix detached from runtime reality
- [ ] no product-local fallback or scan policy

## Execution Plan

### Batch 8.1 - Linux Parity Contract

- [ ] define portable capability, fallback, and sandbox-policy expectations on Linux
- [ ] classify what remains adapter-private after the widened baseline

### Batch 8.2 - Runtime Parity Depth

- [ ] align lifecycle, render, failure, and placement receipts across Linux adapters
- [ ] keep supervisor export and host-edge surfaces on one Linux plugin vocabulary

### Batch 8.3 - Cross-Adapter Proof

- [ ] add focused proofs for Linux plugin parity and sandbox-policy behavior

## Acceptance Criteria

- [ ] Signal has an explicit Linux cross-adapter parity surface
- [ ] sandbox policy remains reusable across the widened Linux adapter set
- [ ] later consumers can rely on one portable Linux plugin vocabulary

## Risks And Mitigations

- Risk: Linux parity work devolves into adapter sprawl.
- Mitigation: freeze one bounded portable contract first.

## Evidence Requirements

- [ ] log each meaningful Linux parity tranche
- [ ] run focused Linux cross-adapter conformance validation
- [ ] record explicit unsupported Linux parity explicitly

## Next Task

Continue `g07.009` by deepening Linux hardware backend portability across ALSA,
JACK, and PipeWire.

