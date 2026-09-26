# Retired backlog: post-g08 repeated-run and environment-matrix depth

Status: triage (non-authoritative)
Source: Contract `071` and the historical `g08.020` closeout in Git history.
Owner: core-product

Triage holds unresolved candidates only. Selection follows the
[plan](../plan.md) and a Queue brief.

## Deferred candidacy

`g08` closed its bounded live-ownership and workflow expansion queue with one
repo-owned integrated acceptance and closeout gate. Deliberately outside that
finished bar:

- broader repeated-run confidence over the bounded integrated acceptance lane;
- richer environment-specific matrices across Linux sessions, devices,
  immersive paths, and preview or workflow mixes;
- stronger shared downstream workflow hardening beyond the current bounded
  closeout-ready substrate.

Constraint: keep any promoted queue inside Signal-owned reusable boundaries.
Promote repeated-run and environment-matrix depth only where the resulting
evidence stays typed, machine-readable, and repo-owned. Keep product-local
controller, browser, immersive-console, certification, and downstream launch
workflows explicitly deferred unless they become reusable shared boundaries
first. Do not reopen backend-local, browser-local, device-private, or
renderer-private policy.

## Promotion condition

Promote only when maintainers want stronger repeated-run confidence, broader
shared environment matrices, or deeper shared downstream workflow hardening
beyond `g08`'s bounded closeout gate.

## Risks (carried over)

- Repeated-run and environment-matrix work can sprawl into expensive
  certification inventory rather than shared bounded evidence.
- Downstream workflow hardening can drift into product-local launch or UX
  policy instead of reusable runtime substrate.
- Broader integrated acceptance can become opaque if it stops emitting typed
  descriptors and collapses back into ad hoc scenario bundles.

## Next check

Review during the next refresh or docs-currentness pass and promote, merge,
or remove it.
