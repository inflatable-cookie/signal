# 021 - g09.011 Demo Coverage Matrix

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.011
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/011-interactive-demo-substrate-manifest-and-operator-conventions.md
Auto-start next card: no

## Objective

Take the final honest `g09.011` substrate seam by freezing the first
crate-to-demo coverage matrix, including explicit deferred status for crates
that do not yet map to a live demo surface.

## Scope

- stay inside docs, demo manifests, and the minimum shared substrate files
  needed for a coverage matrix
- define the first mapping from active crates to demo surfaces or explicit
  deferred status
- keep the matrix tied to the shared substrate and current manifests
- do not widen into full domain demo implementation yet

## Steps

1. Freeze the coverage-matrix seam from `g09.011` and contract `079`.
2. Add the first crate-to-demo coverage matrix surface.
3. Mark unsupported or deferred crates explicitly rather than implying future
   coverage.
4. Record the matrix posture in the active roadmap and substrate docs.
5. Rerun focused docs and repo health validation.

## Acceptance Criteria

- active crates map to a demo surface or explicit deferred status
- deferred coverage is explicit and inspectable
- the matrix is ready for later domain demo milestones without reopening shared
  substrate planning
- focused validation passes

## Evidence Required

- batch log for the next `g09.011` tranche
- validation actually run
- explicit note that full domain demos remain deferred to `g09.012+`

## Stop Conditions

- the batch widens into full runtime, host, plugin, hardware, DSP, or analysis
  demo implementation
- the matrix overclaims live demo coverage that does not yet exist
- the work starts backfilling every historical crate instead of the active
  workspace

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md`.
