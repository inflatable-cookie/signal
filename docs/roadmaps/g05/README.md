# g05 Milestones

Status: complete
Updated: 2026-03-12

## Why this generation matters now

`g04` closed Signal's first stable reusable boundary: runtime/export/plugin
contracts are explicit, consumer conformance is runnable, and the first
release-boundary baseline is host-free and repo-owned.

The next Signal-owned bottleneck is widening that boundary without reopening
host-local ownership:

- backend-neutral plugin breadth beyond the current CLAP-first proof path
- deciding which host convenience APIs become stable shared consumer surfaces
- publication-grade packaging and release receipts beyond changelog plus schema
  descriptions
- longer-running downstream conformance and release automation that stays
  shared rather than app-local

This generation stays inside Signal-owned reusable boundaries:

- no product-specific plugin browser or session UX work
- no consumer-local release orchestration as the source of truth
- no ad hoc host-local reconstruction of runtime or export state
- no backend breadth that bypasses the existing runtime/export/plugin authority

## Dependency order

1. widen backend-neutral plugin capability and adapter breadth first
2. freeze which host convenience APIs are stable shared consumer edges second
3. deepen release packaging and publication-grade receipts on those explicit
   boundaries
4. expand long-running downstream conformance and release automation after the
   boundary and packaging contracts are stronger
5. close the generation with a combined readiness and backlog handoff proof

## Milestone map

- `g05.001` `complete`
  - backend-neutral plugin capability and adapter breadth baseline
- `g05.002` `complete`
  - shared host convenience API and consumer-edge contracts
- `g05.003` `complete`
  - publication-grade packaging manifests and release automation receipts
- `g05.004` `complete`
  - downstream conformance soak and release-acceptance automation
- `g05.005` `complete`
  - generation closeout and promotion gate

## Working rules for this thread

- keep runtime/export/plugin ownership inside Signal crates and typed surfaces
- promote backend breadth only when capability and lifecycle semantics remain
  adapter-neutral at the consumer boundary
- decide host convenience API stability explicitly rather than by leakage from
  current host implementations
- prefer repo-owned packaging, conformance, and release receipts over prose-only
  policy
- keep one active queue and move deferred scope back into backlog when it stops
  being generation-critical

## Next Task

COMPLETE. `g05` closed on 2026-03-13 after `g05.005` finished the combined
readiness gate and explicit backlog handoff. Promote
`docs/roadmaps/backlog/post-g05-publication-promotion-and-shared-acceptance-depth.md`
only when maintainers choose to open the post-`g05` generation.
