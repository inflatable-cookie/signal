# 12-231202 g05.002 Host-Edge Stability Contract Baseline Tranche

Status: complete
Owner: core-product
Related roadmap: `docs/roadmaps/g05/002-shared-host-convenience-api-and-consumer-edge-contracts.md`

## Summary

Completed `g05.002` Batch 2.1 by freezing the first shared host convenience
API stability contract and moving the queue to receipt/export alignment.

## Work Completed

- added `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`
  to classify shared host convenience APIs by stability tier
- promoted `LocalRuntimeHost::new`, `ServerRuntimeHost::new`,
  `RuntimeSupervisorApi`, and `supervisor_report()` into the first shared
  stable host-edge tier
- explicitly kept host-specific summary/report enrichments, scenario boot
  helpers, and local delegated-executor convenience methods outside the stable
  shared boundary for now
- updated the roadmap, contract index, and architecture/reference trail so
  `g05.002` now points at Batch 2.2 rather than leaving the host-edge contract
  question open

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

The stable host-edge tier is now explicit, but it is still narrow. Host-specific
report enrichment and summary surfaces remain asymmetric, and Batch 2.2 still
needs to make the chosen stable edge more inspectable through Signal-owned
receipts or exports before later packaging or conformance work depends on it.

## Next Task

Continue `g05.002` with Batch 2.2 by making the chosen stable host-edge
surfaces inspectable through Signal-owned receipts or exports while keeping
host-specific summaries, boot helpers, and delegated local executor helpers
explicitly unstable.
