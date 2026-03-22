# 018 - Control-Surface And Preview Workflow Acceptance Depth

Status: complete
Owner: core-product
Created: 2026-03-22
Depends on: g08.017
Vision tags: `CONTROL`, `PREVIEW`, `ACCEPTANCE`

## Problem

`g08.017` closes the bounded immersive acceptance seam, but the shared
consumer proof for control-surface workflow and preview-transform workflow
behavior is still fragmented across earlier advanced-hardware,
control-surface-workflow, preview-transform, and preview-device boundaries.

Without one explicit acceptance milestone here, later device or preview work
risks drifting into controller-page UX shells, browser-local queue policy, or
ad hoc rerun coverage that shared consumers cannot rely on.

## Goals

- [ ] freeze one shared acceptance target for control-surface and preview
      workflow behavior
- [ ] keep the acceptance seam grounded in existing runtime-owned advanced
      hardware and preview-transform receipts
- [ ] avoid device-private workflow glue or browser-local preview policy
      becoming the shared proof surface

## Non-Goals

- [ ] no product-local controller page editor or browser queue editor
- [ ] no richer device scripting or preview UX as the shared acceptance surface

## Execution Plan

### Batch 18.1 - Workflow Acceptance Contract

- [x] freeze the shared control-surface and preview workflow acceptance
      contract
- [x] define the mandatory runtime, supervisor, and stable host-edge proof
      spine explicitly

### Batch 18.2 - Acceptance Descriptor And Task

- [x] wire the first repo-owned descriptor and acceptance lane for the shared
      workflow seam
- [x] keep optional device-native and browser-native depth explicit rather than
      folding it into the mandatory shared contract

### Batch 18.3 - Consumer Proof Closure

- [x] prove the widened control-surface and preview workflow acceptance seam
      through shared runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] control-surface and preview workflow acceptance are repo-owned and
      inspectable
- [ ] device-private and browser-local workflow detail stays bounded and typed
- [ ] later integrated acceptance work can build on one explicit shared
      workflow acceptance seam

## Risks And Mitigations

- Risk: workflow acceptance drifts into device-private controller UX or
  browser-local preview queue policy.
- Mitigation: freeze one shared acceptance contract before widening broader
  rerun or integrated depth.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after descriptor/task changes land
- [ ] record the next milestone step explicitly

## Batch 18.1 Outcome

- `g08.018` now has a frozen shared acceptance contract in
  `docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md`
  instead of leaving grouped controller and preview workflow proof fragmented
  across the existing advanced-hardware and preview-transform seams
- the later shared lane is now required to compose through public runtime
  receipts, supervisor export, and both stable host edges rather than
  device-private page logic or browser-local queue policy
- grouped descriptor, Effigy acceptance lane, and broader device-native or
  browser-native workflow depth remain explicitly deferred until later
  `g08.018` batches

## Batch 18.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.control-preview-workflow-acceptance-lane` descriptor instead
  of leaving grouped controller and preview workflow proof fragmented across
  the isolated advanced-hardware and preview-transform boundaries
- Effigy now owns one runnable
  `effigy acceptance:control-preview-workflow-acceptance-lane` task that
  composes the bounded proof spine into one shared lane while keeping broader
  device-native and browser-native workflow reruns explicitly non-blocking
- `g08.018` now has a real grouped acceptance surface, and the remaining work
  is the final consumer-proof closure rather than more policy setup

## Batch 18.3 Outcome

- one repo-owned supervisor export proof now demonstrates that control-surface
  workflow, advanced-feedback, preview-device policy, and preview-workflow
  receipts are consumable together instead of only through the grouped
  descriptor and the isolated boundary tasks
- `effigy acceptance:control-preview-workflow-acceptance-lane` now composes
  the grouped descriptor, grouped export proof, and the existing advanced-
  hardware and preview-transform proof spine into one reusable shared
  acceptance lane
- `g08.018` is now complete, and the next `g08` queue is integrated
  live-ownership and workflow acceptance depth

## Completion

`g08.018` is complete. Shared control-surface and preview workflow acceptance
now has a frozen contract, a repo-owned descriptor, a runnable grouped lane,
and one explicit grouped consumer proof.

## Next Task

Continue `g08.019` with Batch 19.1 by freezing the shared integrated live-
ownership and workflow acceptance contract on top of the closed Linux live,
device workflow, immersive, and control-preview workflow acceptance seams.
