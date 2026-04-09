# 012 - g09.009 Semantic Confidence Calibration

Status: ready
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.009
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/009-dsp-fidelity-and-semantic-analysis-realism-uplift.md
Auto-start next card: no

## Objective

Continue `g09.009` by making semantic confidence posture more explicit: keep
the frozen tone/noise/pulse corpus and explainable tag evidence, but tighten
how confidence is derived and reported so the crate is not still relying on one
opaque heuristic blend over margin and embedding activity.

## Scope

- stay inside `crates/signal-analysis-embed`
- keep the existing frozen corpus and explainable tag evidence
- refine or separate confidence calibration logic so its posture is explicit
  and testable
- add focused tests that prove the frozen corpus still has interpretable
  confidence ordering after the calibration change
- do not widen into rhythm or demo work

## Steps

1. Extract or clarify the semantic-confidence calculation posture in
   `signal-analysis-embed`.
2. Make the confidence policy more explicit and separately testable.
3. Add focused corpus-backed confidence-ordering assertions for the frozen
   semantic cases.
4. Rerun the focused semantic validation surface plus repo health.

## Acceptance Criteria

- semantic confidence is no longer an opaque inline heuristic
- frozen semantic cases have explicit confidence-ordering expectations
- focused validation passes

## Evidence Required

- batch log for the next `g09.009` tranche
- validation actually run
- explicit note if any further semantic tuning remains deferred after this
  confidence pass

## Stop Conditions

- the batch starts redesigning the whole model stack instead of tightening
  confidence posture
- the work broadens into rhythm or demo proof rather than semantic calibration

## Next Task

Implement this semantic confidence calibration batch, then reassess whether
`g09.009` is ready to hand off toward `g09.010`.
