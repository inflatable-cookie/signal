# 020 - g09.011 Demo Launch And Evidence Conventions

Status: ready
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.011
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/011-interactive-demo-substrate-manifest-and-operator-conventions.md
Auto-start next card: no

## Objective

Take the next honest `g09.011` substrate seam by freezing how official demos
are launched and how their operator evidence is captured, without widening into
full domain demo breadth or coverage-matrix backfill.

## Scope

- stay inside docs, Effigy task surfaces, and the minimum shared demo-substrate
  files needed for launch and evidence conventions
- define the required launch-command posture for official demos
- define receipt/log capture and expected human-check conventions
- define how operator notes attach to scenario manifests
- do not widen into full runtime, host, plugin, hardware, DSP, or analysis
  demo implementation yet

## Steps

1. Freeze the launch and evidence convention seam from `g09.011` and contract
   `079`.
2. Add the minimum shared files or templates for operator notes and evidence
   receipts.
3. Add or adjust repo-owned Effigy task posture so official demo launch and
   evidence conventions are explicit.
4. Record the conventions in the active roadmap and substrate docs.
5. Rerun focused docs and repo health validation.

## Acceptance Criteria

- launch-command posture for official demos is explicit and current
- evidence and operator-note conventions are explicit and reusable
- the next demo tranche can implement domain demos without reopening shared
  launch/evidence planning
- focused validation passes

## Evidence Required

- batch log for the next `g09.011` tranche
- validation actually run
- explicit note that full domain demos and coverage matrix remain deferred to
  later milestones

## Stop Conditions

- the batch widens into actual domain demo implementation
- the evidence model still requires fresh planning judgment after the tranche
- the work turns into downstream app workflow design instead of shared demo
  substrate rules

## Next Task

Implement this demo launch-and-evidence batch, then continue `g09.011`
through the coverage-matrix seam if the shared substrate is truly settled.
