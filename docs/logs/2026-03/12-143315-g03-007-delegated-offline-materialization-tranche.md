# g03.007 - Delegated Offline Materialization Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed Batch 7.6 in `signal-runtime`. Delegated offline plugin execution no
longer stops at an in-memory receipt fold-in. The runtime now routes delegated
receipt application back through the same offline artifact/report
materialization path, so report export and manifest delivery stay aligned with
the delegated handoff state.

## Shipped

- refactored offline artifact/report writing into one runtime-owned delivery
  materialization helper reused by both initial offline render export and
  delegated receipt application
- changed delegated receipt application to re-materialize the same offline
  delivery bundle when an artifact root is present instead of leaving on-disk
  report state stale
- extended runtime-owned offline report JSON with delegated execution request
  and receipt detail so delegated-stage export can be inspected without host
  supervisor parsing
- added focused proof that delegated receipt application updates both the
  in-memory manifest bundle and the rewritten runtime-owned report export

## Deferred

- delegated execution still does not carry real host-rendered stage outputs
  back into offline audio finalization; today it only materializes the request
  and receipt handoff plus aligned export metadata
- graph projection still requires at least one stage per node, so true
  host-only delegated stage shapes remain a later executor-bridge proof
- offline parity for host-only plugin stages therefore still depends on a
  future delegated executor merge contract

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.007` with Batch 7.7 by defining the delegated executor
output/merge contract and by proving one delegated executor fixture can feed
runtime-owned finalization through the same delivery bundle before opening
`g03.008`.
