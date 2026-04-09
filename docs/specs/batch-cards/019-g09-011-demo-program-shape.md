# 019 - g09.011 Demo Program Shape

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.011
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/011-interactive-demo-substrate-manifest-and-operator-conventions.md
Auto-start next card: no

## Objective

Take the first honest `g09.011` substrate seam by defining the demo program
shape: where demo binaries live, how scenarios are declared, and what one
machine-readable manifest entry must contain before any domain demo breadth is
implemented.

## Scope

- stay inside docs, repo task surfaces, and the minimum shared demo-substrate
  code or file layout needed to freeze the program shape
- define one canonical location and naming rule for demo binaries or scenario
  bundles
- define the initial manifest schema for covered crates, covered scenarios,
  exclusions, launch command, and expected human checks
- choose the first dedicated-versus-shared demo grouping rule for later
  milestones
- do not widen into full domain demos, polished UI, or complete coverage
  matrices yet

## Steps

1. Freeze the shared demo program shape from `g09.011` roadmap and contract
   `079`.
2. Define the substrate location and manifest schema in repo-owned surfaces.
3. Add the minimum repo-owned task or entrypoint placeholder needed to make the
   shape executable later.
4. Record the dedicated-versus-shared grouping rule for future domain demos.
5. Rerun focused docs and repo health validation.

## Acceptance Criteria

- demo location and manifest schema are explicit and current
- the shared grouping rule for later demo milestones is explicit
- the substrate is ready for later domain demo execution without fresh shape
  decisions
- focused validation passes

## Evidence Required

- batch log for the first `g09.011` tranche
- validation actually run
- explicit note that domain demos remain deferred to `g09.012+`

## Stop Conditions

- the batch widens into full demo implementation or coverage-matrix backfill
- the manifest shape still requires fresh planning judgment after the tranche
- the work starts inventing product-local UI or operator workflows beyond the
  shared substrate

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/020-g09-011-demo-launch-and-evidence-conventions.md`.
