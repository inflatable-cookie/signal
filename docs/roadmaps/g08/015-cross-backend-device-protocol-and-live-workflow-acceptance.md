# 015 - Cross-Backend Device Protocol And Live Workflow Acceptance

Status: active
Owner: core-product
Created: 2026-03-22
Depends on: g08.014
Vision tags: `DEVICE`, `BACKEND`, `ACCEPTANCE`

## Problem

`g08.014` closes the bounded live external MIDI ownership seam, but the shared
consumer proof for cross-backend device protocol and live workflow behavior is
still fragmented across earlier external MIDI, controller, advanced-hardware,
and Linux live-ownership boundaries.

Without one explicit acceptance milestone here, later device protocol work
risks drifting into backend-local endpoint policy, host-private controller
workflows, or ad hoc rerun coverage that shared consumers cannot rely on.

## Goals

- [ ] freeze one shared acceptance target for cross-backend device protocol and
      live workflow behavior
- [ ] keep the acceptance seam grounded in existing runtime-owned external
      MIDI, controller-expression, control-surface, advanced-hardware, and
      live ownership receipts
- [ ] avoid backend-local device policy or product-local workflow glue becoming
      the shared proof surface

## Non-Goals

- [ ] no product-local controller UX or live performance scene automation
- [ ] no backend-specific patchbay, routing console, or rehearsal tool as the
      shared acceptance surface

## Execution Plan

### Batch 15.1 - Device Workflow Acceptance Contract

- [x] freeze the shared cross-backend device protocol and live workflow
      acceptance contract
- [x] define the mandatory runtime, supervisor, and stable host-edge proof
      spine explicitly

### Batch 15.2 - Acceptance Descriptor And Task

- [ ] wire the first repo-owned descriptor and acceptance lane for the shared
      device workflow seam
- [ ] keep optional backend-specific depth explicit rather than folding it into
      the mandatory shared contract

### Batch 15.3 - Consumer Proof Closure

- [ ] prove the widened device workflow acceptance seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] cross-backend device protocol and live workflow acceptance are repo-owned
      and inspectable
- [ ] backend-local or host-local device workflow detail stays bounded and typed
- [ ] later Linux and integrated acceptance work can build on one explicit
      device-protocol acceptance seam

## Risks And Mitigations

- Risk: live device workflow acceptance drifts into backend-local endpoint
  policy or product-specific controller glue.
- Mitigation: freeze one shared acceptance contract before widening further
  backend or workflow depth.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after descriptor/task changes land
- [ ] record the next milestone step explicitly

## Batch 15.1 Outcome

- `g08` now has a frozen shared device-workflow acceptance contract in
  `docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md`
  instead of leaving grouped live device protocol proof fragmented across the
  external MIDI, controller, control-surface, advanced-hardware, and live
  ownership seams
- the shared acceptance lane is now required to compose through public
  runtime receipts, supervisor export, and both stable host edges rather than
  backend-local endpoint policy or host-private workflow glue
- the grouped descriptor, Effigy acceptance lane, and broader advisory versus
  deferred device-depth policy remain explicitly deferred until Batch 15.2
  and Batch 15.3

## Next Task

Continue `g08.015` with Batch 15.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared cross-backend device protocol and live
workflow seam while keeping backend-specific depth explicit and non-blocking.
