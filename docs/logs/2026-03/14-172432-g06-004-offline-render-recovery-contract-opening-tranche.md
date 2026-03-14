# 2026-03-14 17:24:32 - g06.004 offline render recovery contract opening tranche

## What changed

- opened contract `015` to freeze the first shared offline-render recovery and
  resumability vocabulary on top of the completed interruption contract
- anchored offline render recovery to runtime-owned request identity,
  checkpoints, execution progress, cancellation, queue orchestration, and
  manifest or artifact alignment instead of host-local queue or retry models
- defined how `Resumable`, `Restartable`, `Recoverable`, `Terminal`, and
  `Rebindable` apply to offline render sessions without creating a competing
  render-only taxonomy
- marked Batch 4.1 complete in `g06.004` and moved the active next step to
  Batch 4.2 runtime session-depth work

## Evidence

- `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
- `docs/roadmaps/g06/004-offline-render-execution-recovery-and-resumability-depth.md`
- `docs/contracts/README.md`
- `docs/roadmaps/g06/README.md`
- `docs/roadmaps/README.md`
- `docs/roadmaps/generation-index.md`
- `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- deeper checkpoint survival rules across runtime restart or process restart
  still belong to Batch 4.2 and 4.3
- dedicated public host-edge render continuity descriptors or acceptance tasks
  are still deferred until the runtime receipt family is widened further
- distributed queue ownership and remote job orchestration remain out of scope
  for this milestone

## Next Task

Continue `g06.004` with Batch 4.2 by deepening runtime render-session
receipts, checkpoint survival, and rebind surfaces while keeping manifest,
artifact, and purge semantics coherent with the new recovery contract.
