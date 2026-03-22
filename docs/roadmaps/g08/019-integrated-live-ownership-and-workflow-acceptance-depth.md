# 019 - Integrated Live-Ownership And Workflow Acceptance Depth

Status: complete
Owner: core-product
Created: 2026-03-22
Depends on: g08.018
Vision tags: `LINUX`, `DEVICE`, `PREVIEW`, `IMMERSIVE`, `ACCEPTANCE`

## Problem

`g08.018` closes the grouped control-surface and preview workflow acceptance
seam, but the broader `g08` consumer proof is still split across separate
Linux live, device workflow, immersive, and control-preview acceptance lanes.

Without one integrated milestone here, later closeout work risks treating
those shared lanes as parallel checklists instead of one coherent live-
ownership and workflow acceptance story.

## Goals

- [ ] freeze one shared integrated acceptance target for the widened `g08`
      live-ownership and workflow substrate
- [ ] keep the integrated seam grounded in existing repo-owned grouped lanes
- [ ] avoid backend-local, device-private, browser-local, or renderer-private
      glue becoming the integrated proof surface

## Non-Goals

- [ ] no distro certification matrix or renderer-vendor certification program
- [ ] no product-local controller, browser, or immersive console workflow

## Execution Plan

### Batch 19.1 - Integrated Acceptance Contract

- [x] freeze the shared integrated live-ownership and workflow acceptance
      contract
- [x] define the required grouped lane spine explicitly

### Batch 19.2 - Integrated Descriptor And Task

- [x] wire the first repo-owned integrated descriptor and acceptance lane
- [x] keep richer repeated-run and environment-specific depth advisory

### Batch 19.3 - Integrated Consumer Proof Closure

- [x] prove the widened integrated acceptance seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] integrated live-ownership and workflow acceptance are repo-owned and
      inspectable
- [ ] backend-local, device-private, browser-local, and renderer-private
      workflow detail stays bounded
- [ ] `g08.020` closeout can build on one explicit integrated acceptance seam

## Risks And Mitigations

- Risk: integrated acceptance collapses back into a checklist of unrelated lane
  descriptors.
- Mitigation: freeze one integrated contract before wiring the grouped
  closeout-facing lane.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after descriptor/task changes land
- [ ] record the next milestone step explicitly

## Batch 19.1 Outcome

- `g08.019` now has a frozen integrated acceptance contract in
  `docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md`
  instead of leaving the broader `g08` closeout-facing proof split across four
  parallel grouped lanes only
- the later integrated lane is now required to compose through public runtime
  receipts, supervisor export, and both stable host edges rather than
  backend-local, device-private, browser-local, or renderer-private
  coordination glue
- integrated descriptor, Effigy acceptance lane, and broader repeated-run or
  closeout-adjacent depth remain explicitly deferred until later `g08.019`
  batches

## Batch 19.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane`
  descriptor instead of leaving the broader `g08` integrated claim split
  across four grouped lane descriptors only
- Effigy now owns one runnable
  `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  task that composes the closed Linux live, device workflow, immersive, and
  control-preview workflow lanes into one repo-owned integrated seam while
  keeping repeated-run and environment-specific depth explicitly non-blocking
- the remaining work is now the final grouped consumer-proof closure rather
  than more descriptor or task setup

## Batch 19.3 Outcome

- one repo-owned supervisor export proof now demonstrates that Linux live
  ownership, device workflow, immersive render and monitoring, and
  control-preview workflow receipts are consumable together instead of only
  through the grouped descriptor and grouped Effigy lane
- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  now composes the grouped export proof together with the four grouped lanes
  and the integrated descriptor, closing the shared integrated acceptance seam
- `g08.019` is now complete, and `g08.020` is the next queue for generation
  closeout and downstream workflow readiness

## Completion

`g08.019` is complete. Signal now has one explicit shared integrated
live-ownership and workflow acceptance seam that downstream closeout work can
build on without rediscovering how Linux live, device workflow, immersive,
and preview workflow evidence fit together.

## Next Task

Continue `g08.020` with Batch 20.1 by freezing the shared generation closeout
and downstream workflow readiness contract on top of the closed `g08.019`
integrated acceptance seam.
