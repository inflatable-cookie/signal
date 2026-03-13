# 013 - Plugin Preset, ARA Context, State Interchange, And Portable Recall Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.011, g06.012
Vision tags: `PLUGINS`, `STATE`, `INTERCHANGE`

## Problem

Signal already owns plugin recall and delegated execution surfaces, but broader
adapter breadth and product portability both need a clearer reusable answer for
state interchange, preset-family meaning, ARA-capable clip/plugin context, and
portable recall constraints.

## Goals

- [ ] define portable plugin state and preset interchange semantics where
  they are actually reusable
- [ ] define the first reusable ARA-capable plugin context vocabulary where
  clip/region/document metadata must cross the Signal runtime boundary
- [ ] align recall ownership, exported state receipts, and cross-adapter
  fallback behavior
- [ ] support later product recall and migration work without host-local blobs
  becoming the only practical path

## Non-Goals

- [ ] no product-specific preset browser or tagging UX
- [ ] no promise of lossless interchange where adapters cannot support it
- [ ] no full product-local clip editor workflow or arrangement policy

## Execution Plan

### Batch 13.1 - Interchange And ARA Contract

- [ ] define portable preset/state interchange vocabulary, ARA context
  descriptors, and fallback classes
- [ ] record what remains adapter-private or non-portable explicitly

### Batch 13.2 - Runtime Recall Depth

- [ ] deepen runtime-owned recall/export surfaces to carry the new interchange
  and ARA-context meaning
- [ ] keep render/delegation/host-edge receipts aligned to the same state truth

### Batch 13.3 - Portability Proof

- [ ] add focused proofs for portable versus non-portable recall outcomes and
  bounded ARA-context transfer across the widened adapter set

## Acceptance Criteria

- [ ] Signal has explicit portable plugin state/preset interchange meaning and
  the first bounded ARA-capable context contract
- [ ] products can observe portable versus non-portable recall outcomes clearly
- [ ] state portability does not reopen host-local ownership

## Risks And Mitigations

- Risk: portability claims overpromise across adapters.
- Mitigation: freeze supported, degraded, and unsupported interchange classes explicitly.
- Risk: ARA planning drifts into product clip-edit workflows before the runtime
  boundary is clear.
- Mitigation: keep this milestone on runtime-owned clip/region/document context
  descriptors and plugin capability meaning only.
- Risk: products keep using opaque blobs because the portable path is unclear.
- Mitigation: require typed runtime receipts for portability outcomes.

## Evidence Requirements

- [ ] log each meaningful preset/recall tranche
- [ ] run focused portability validation for runtime-owned recall/export surfaces
- [ ] record explicit deferred preset breadth that remains out of scope

## Next Task

Continue `g06.014` by pushing hardware supervision depth further down into
Signal instead of leaving restart logic to products.
