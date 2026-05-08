# 2026-04-09 - g09.009 Semantic Calibration Baseline Tranche

## Summary

Completed the first semantic-calibration batch in the strict `g09.009` lane.

## Implementation

- added explainable `SemanticTagEvidence` and emitted it for each built-in tag
- added `top_tag_label` to semantic diagnostics
- added machine-readable semantic calibration report types and a frozen
  calibration report surface for the tone, noise, and pulse reference cases
- widened the semantic tests so the frozen corpus asserts expected top-tag,
  evidence-driver, and confidence posture explicitly

## Validation

- `cargo test -p signal-analysis-embed`
- `cargo check -p signal-dsp-resample`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Reassessment

One honest `g09.009` seam remains before handing off to `g09.010`: make the
semantic confidence policy itself more explicit and testable now that the
corpus and explainable evidence are frozen.

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/012-g09-009-semantic-confidence-calibration.md`.
