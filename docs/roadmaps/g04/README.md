# g04 Milestones

Status: complete
Updated: 2026-03-12

## Why this generation matters now

`g03` proved that Signal can own a credible engine substrate: routed mixer
topology, diagnostics, automation playback, warp/render, plugin-chain
execution, offline render, and runtime hardening all now exist as reusable
engine work rather than app-local glue.

The next Signal-owned bottleneck is different. The repo now needs a stronger
multi-consumer product surface:

- explicit public crate and schema boundaries
- deeper multicore and anticipative execution policy
- signal-owned orchestration for non-realtime and deferred runtime work
- broader hardware and plugin portability without leaking host-local policy
- release-ready conformance surfaces for Loophole, Finch, and later consumers

This generation stays inside Signal-owned reusable boundaries:

- no product workflow ownership
- no Pulse/Aura/Spark-local session semantics
- no app-specific export UX or orchestration plans
- no cross-repo planning duplication in Chorus

## Dependency order

1. freeze the reusable contract and crate maturity boundary first
2. deepen multicore scheduling and anticipative execution on that explicit
   contract
3. make deferred/background runtime work a first-class Signal concern
4. expand hardware portability and clock-domain depth on the stronger
   scheduling/orchestration substrate
5. widen plugin backend and delegation contracts without reintroducing
   host-local ownership
6. close the generation with consumer conformance and release packaging

## Milestone map

- `g04.001` `complete`
  - crate maturity, public contracts, and schema-freeze baseline
- `g04.002` `complete`
  - multicore graph scheduling and anticipative execution depth
- `g04.003` `complete`
  - runtime work orchestration and deferred service policy
- `g04.004` `complete`
  - hardware backend portability and clock-domain boundary depth
- `g04.005` `complete`
  - plugin backend breadth and host-neutral delegation contracts
- `g04.006` `complete`
  - consumer conformance, export stability, and release packaging

## Working rules for this thread

- keep reusable behavior in Signal crates, contracts, and host-neutral export
  surfaces rather than product repos
- prefer typed crate/API/schema boundaries over narrative-only “intended use”
  docs
- keep realtime-safe scheduling work separate from orchestration or packaging
  helpers that can run off the critical path
- widen backend/consumer breadth only after the public boundary is explicit
- prove new reusable boundaries with focused fixtures, export checks, or
  consumer-facing examples before calling them stable

## Next Task

COMPLETE. `g04` closed on 2026-03-12 after `g04.006` finished the combined
consumer-conformance, release-boundary, and closeout proof. The next likely
queue is recorded in
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`.
