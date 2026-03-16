# 2026-03-16 15:14:04 UTC - g06.019 Fault-Injection Contract Opening Tranche

## Summary

Opened `g06.019` by freezing the first shared fault-injection harness and
multi-backend acceptance contract. The batch defines which integrated scenario
families belong to the bounded `g06` acceptance lane and separates required
evidence from advisory and deferred depth before implementation work begins.

## Work completed

- added the new integrated acceptance contract:
  - `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
- recorded the Batch 19.1 outcome in the active roadmap:
  - `docs/roadmaps/g06/019-fault-injection-harnesses-and-multi-backend-acceptance-depth.md`
- updated the contract index and shared next-task pointers:
  - `docs/contracts/README.md`
  - `docs/roadmaps/g06/README.md`
  - `docs/roadmaps/README.md`
  - `docs/roadmaps/generation-index.md`
- refreshed the architecture reference to reflect the frozen integrated
  acceptance policy:
  - `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- Batch 19.1 freezes policy only; it does not yet add a new integrated
  harness descriptor or Effigy acceptance lane
- long-session soak thresholds and promotion gates still belong to `g06.020`
- unstable broader server-host recovery-overlap scenarios remain explicitly
  deferred until the bounded integrated lane is real

## Next Task

Continue `g06.019` with Batch 19.2 by implementing the first integrated
fault-injection harness descriptor and Effigy acceptance lane on top of the
required versus advisory policy frozen here.
