# 011 - g09.009 Semantic Calibration Baseline

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.009
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/009-dsp-fidelity-and-semantic-analysis-realism-uplift.md
Auto-start next card: no

## Objective

Continue `g09.009` by freezing the first real semantic-calibration seam:
define one bounded corpus-backed comparison surface for
`signal-analysis-embed`, publish explicit expected top-tag and confidence
posture for the frozen examples, and stop relying only on hand-tuned weights
without explainable evidence.

## Scope

- stay inside `crates/signal-analysis-embed`
- add one stable corpus-backed calibration/report surface for the built-in
  semantic model
- make expected tag ordering and confidence posture explicit for the frozen
  example set
- do not widen into broader model/platform work or demo substrate

## Steps

1. Choose the bounded semantic corpus/report surface already closest to stable
   in `signal-analysis-embed`.
2. Promote that surface from ad hoc example output into explicit calibration
   evidence with frozen expected posture.
3. Add focused tests that prove top-tag and confidence behavior remains
   interpretable for the frozen cases.
4. Rerun the focused semantic and dependent validation surface plus repo
   health.

## Acceptance Criteria

- semantic-tag output has corpus-backed explainable evidence, not only heuristic
  weights
- the frozen semantic cases encode expected top-tag and confidence posture
- focused validation passes

## Evidence Required

- batch log for the next `g09.009` tranche
- validation actually run
- explicit note if deeper confidence calibration remains deferred after the
  baseline is frozen

## Stop Conditions

- the batch starts designing a new model platform or remote inference system
- the work widens into demo or downstream UX instead of semantic evidence

## Next Task

Continue the active strict lane from
`docs/specs/batch-cards/012-g09-009-semantic-confidence-calibration.md`.

## Outcome

`signal-analysis-embed` now publishes explainable semantic evidence instead of
only heuristic scores and confidence values. Each top-ranked tag carries a
stable primary and supporting driver, diagnostics now record the top emitted
label, and the frozen synthetic semantic corpus has a machine-readable
calibration report with explicit top-tag and confidence posture for the tone,
noise, and pulse reference cases.

## Validation Run

- `cargo test -p signal-analysis-embed`
- `cargo check -p signal-dsp-resample`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`
