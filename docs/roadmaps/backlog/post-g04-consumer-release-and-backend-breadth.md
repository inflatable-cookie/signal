# Backlog: Post-g04 Consumer, Release, And Backend Breadth

Status: backlog
Priority: medium
Estimated effort: multi-batch generation
Source: g04.006

## Problem

`g04` closed the first stable reusable Signal boundary, but several
deliberately deferred scopes still sit outside that promise: broader non-CLAP
plugin backend breadth, host convenience API stabilization, publication-grade
packaging, and longer-running downstream conformance automation.

## Proposed approach

Open the next generation only when maintainers want to widen the
consumer-facing surface beyond the current runtime/export/plugin baseline. Keep
the next queue inside Signal-owned reusable boundaries:

- promote broader backend-neutral plugin and capability coverage only where the
  runtime/export boundary can stay authoritative
- decide whether host convenience APIs should become stable shared product
  surfaces or remain explicitly unstable
- deepen release packaging from changelog plus host-free boundary descriptions
  into publication-grade receipts, manifests, and automation
- add longer-running downstream conformance and release acceptance fixtures
  only when they can stay shared and repo-owned rather than app-local

## Promotion trigger

Promote this backlog item when at least one of the following becomes true:

- maintainers want to support plugin backend breadth beyond the current
  CLAP-first conformance boundary
- Signal needs a stronger publication or packaging contract than the current
  changelog plus host-free description baseline
- a downstream consumer such as Loophole or Finch needs broader shared
  conformance automation than the current focused matrix

## Success criteria

- [ ] the next generation names which deferred `g04` scopes are now promoted
- [ ] the new queue keeps runtime/export/plugin ownership inside Signal crates
- [ ] release, conformance, and backend breadth work do not reintroduce
  host-local reconstruction or consumer-specific policy

## Risks

- release/publishing work can sprawl into distribution workflow detail that
  does not belong in Signal
- backend breadth can reopen ownership boundaries that `g04` intentionally
  froze
- longer-running conformance can become expensive integration sprawl instead of
  shared contract proof

## Next Task

Promote this item only when maintainers choose to open the post-`g04`
generation and can name which of the deferred breadth areas now justify active
roadmap work.
