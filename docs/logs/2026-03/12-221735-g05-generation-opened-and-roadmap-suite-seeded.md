# g05 Generation Opened And Roadmap Suite Seeded

Date: 2026-03-12
Scope: `docs/roadmaps/g05/`

## Summary

Opened the next independent Signal generation from the promoted post-`g04`
backlog item and seeded the first roadmap suite for backend breadth, host-edge
contracts, publication-grade packaging, and broader downstream conformance.

## What changed

- marked `g05` as the active Signal roadmap generation in
  `docs/roadmaps/generation-index.md`
- updated `docs/roadmaps/README.md`, `docs/roadmaps/g04/README.md`,
  `docs/contracts/README.md`, and
  `docs/architecture/graph-runtime-feature-reference.md` so the local queue
  points into `g05`
- promoted
  `docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`
  into an opened generation seed
- added `docs/roadmaps/g05/README.md` with the new generation rationale,
  dependency order, and milestone map
- added the first `g05` milestone suite:
  - `g05.001` backend-neutral plugin capability and adapter breadth baseline
  - `g05.002` shared host convenience API and consumer-edge contracts
  - `g05.003` publication-grade packaging manifests and release automation receipts
  - `g05.004` downstream conformance soak and release-acceptance automation
  - `g05.005` generation closeout and promotion gate

## Why this queue

`g04` closed the first credible shared-project boundary, but the next Signal
owned bottleneck is widening that boundary without losing authority: broader
backend-neutral plugin breadth, explicit host-edge stability, stronger release
receipts, and longer-running consumer/release confidence all still need a
reusable Signal-owned answer.

## Validation

- `effigy health --repo .`
- `git diff --check`

## Next

Start with `g05.001` and define the first backend-neutral plugin capability
and adapter-breadth contract before widening host-edge or packaging claims.
