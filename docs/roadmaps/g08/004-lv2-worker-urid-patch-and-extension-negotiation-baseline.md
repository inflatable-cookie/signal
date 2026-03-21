# 004 - LV2 Worker, URID, Patch, And Extension-Negotiation Baseline

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.003
Vision tags: `LINUX`, `LV2`, `PLUGIN`

## Problem

`g08.003` closes the bounded live Linux ownership and PipeWire/ALSA parity
lane, but richer LV2 worker, URID, patch, and extension-negotiation meaning is
still outside shared runtime truth. Without that boundary, deeper Linux-native
LV2 work will drift back into adapter-private feature tables, host-local
negotiation policy, or daemon-adjacent callback reasoning.

## Goals

- [x] freeze runtime-owned LV2 worker, URID, patch, and extension-negotiation meaning
- [x] expose one bounded LV2 extension substrate across shared runtime and stable host edges
- [x] keep adapter-private feature tables and Linux-native callback detail additive rather than authoritative

## Non-Goals

- [ ] no full LV2 UI, atom-schema, or custom extension matrix here
- [ ] no product-local plugin inspector, patch browser, or extension UX

## Execution Plan

### Batch 4.1 - LV2 Extension-Negotiation Contract

- [x] freeze runtime-owned LV2 worker, URID, patch, and extension-negotiation meaning
- [x] define shared runtime versus adapter-private authority explicitly

### Batch 4.2 - Runtime LV2 Extension Baseline

- [x] materialize the first runtime-owned LV2 worker, URID, patch, and extension-negotiation receipts
- [x] align stable host-edge export with the same LV2 extension model

### Batch 4.3 - Consumer Proof

- [x] prove the widened LV2 extension seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] LV2 worker, URID, patch, and extension-negotiation posture is runtime-owned and inspectable
- [x] adapter-private feature-table and callback detail stays bounded and typed
- [x] later Linux plugin and workflow work can build on one explicit LV2 extension authority line

## Risks And Mitigations

- Risk: LV2 extension depth drifts into adapter-private feature tables or host-local negotiation policy.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 4.1 Outcome

Batch 4.1 freezes the bounded LV2 extension-negotiation seam in
`docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`.
That contract layers worker posture, URID negotiation posture, patch exchange
posture, and extension-negotiation summary on top of the closed LV2 baseline
and live Linux ownership seams instead of inventing a competing host-local
feature-policy shell.

It now makes the authority line explicit:

- `038` remains the bounded LV2 lifecycle and Linux-native baseline instead of
  being reopened as a generic extension matrix
- `039` remains the Linux cross-adapter parity and sandbox-policy baseline, so
  guarded versus private LV2 depth still composes through the same shared
  Linux vocabulary
- `052`, `053`, and `054` remain the authority for live Linux backend
  ownership and coordination, so backend session policy cannot be reclassified
  as LV2 negotiation truth
- Batch 4.2 now has one bounded contract target for runtime-owned LV2 worker,
  URID, patch, and extension-negotiation receipts before public proof widens
  in Batch 4.3

## Batch 4.2 Outcome

Batch 4.2 turns the frozen LV2 extension contract into a reusable runtime-owned
receipt family.

- `signal-runtime` now exports typed LV2 worker, URID, patch, and extension
  negotiation posture through one `RuntimeLv2ExtensionSnapshot` surface instead
  of leaving that meaning in adapter-private feature tables or host summaries
- the widened receipt composes from runtime-owned discovery and lifecycle truth,
  so guarded and unavailable outcomes stay visible without inventing a second
  Linux or plugin lifecycle model
- stable host-edge surfaces now export the same LV2 extension snapshot instead
  of reconstructing negotiation posture from host-local adapter detail
- Batch 4.3 can now close the seam with a bounded consumer proof and, if it is
  warranted, a repo-owned acceptance descriptor on top of the same receipt
  family

## Batch 4.3 Outcome

Batch 4.3 closes `g08.004` by widening the existing shared LV2 consumer
boundary to the new extension seam instead of creating a second overlapping LV2
acceptance story.

- the repo-owned `lv2-boundary` descriptor now points at the LV2 worker, URID,
  patch, and extension-negotiation contract instead of the older baseline-only
  contract
- the acceptance lane now requires public runtime proof plus stable local and
  server host-edge proofs for the same runtime-owned LV2 extension snapshot
- supervisor-side machine-readable boundary output now describes the bounded
  extension seam directly, so consumers can inspect the proof surface without
  adapter-private reconstruction
- `g08.004` is complete, and later Linux plugin depth can build on one explicit
  LV2 extension authority line rather than reopening worker or patch policy in
  host code

## Next Task

Open `g08.005` with Batch 5.1 by freezing the first runtime-owned complex
plugin pin-matrix and dynamic bus-negotiation contract on top of the closed
LV2 extension, Linux parity, and live backend seams.
