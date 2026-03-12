# g03.007 - Delegated Executor Outcome Bridge Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed Batch 7.7 in `signal-runtime`. The offline delegated execution seam
now carries not just stage receipts but mergeable output payloads for main
mix, stem, and freeze results, and the runtime can feed those delegated
outputs back through the same finalization and delivery-materialization path.

## Shipped

- added runtime-owned delegated executor merge and outcome DTOs so later host
  adapters can return both execution receipt state and replacement rendered
  outputs without inventing a second export contract
- added runtime-owned merge validation that checks request identity plus audio
  buffer compatibility before delegated outputs are accepted into offline
  finalization
- refreshed offline render summaries, peak/RMS metadata, and rendered-frame
  accounting after delegated merges land so runtime-owned finalization stays
  authoritative
- reused the existing artifact/report materialization path after delegated
  outcome application so rewritten delivery output reflects the merged audio
  state
- added a focused delegated executor fixture proving merged main-mix, stem,
  and freeze outputs flow through runtime finalization, artifact writing, and
  report export without a parallel offline bundle

## Deferred

- there is still no concrete host-local or server adapter invoking this
  runtime-owned delegated executor contract end-to-end
- delegated host-only plugin parity is therefore still demonstrated through a
  runtime fixture rather than a real host execution bridge
- the runtime still does not own a true host-only graph node shape; delegated
  stage handling continues to project through the existing plugin boundary
  contracts

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.007` with Batch 7.8 by wiring one concrete delegated executor
adapter against the runtime-owned request/outcome contract and proving it can
round-trip through runtime preparation, merge, and finalization before opening
`g03.008`.
