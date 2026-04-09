# 010 - g09.009 Resampler Proof And Benchmark Surface

Status: ready
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.009
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/009-dsp-fidelity-and-semantic-analysis-realism-uplift.md
Auto-start next card: no

## Objective

Continue `g09.009` by proving the new resampler quality posture explicitly:
add one focused benchmark or artifact-comparison surface that records the
difference between the low-quality and `BandLimited` paths so the crate does
not overclaim fidelity through enum names alone.

## Scope

- stay inside `crates/signal-dsp-resample`
- add a machine-readable or frozen test-friendly comparison surface for the
  quality tiers
- prove the `BandLimited` mode improves a bounded artifact measure compared
  with interpolation-only modes
- do not widen into semantic calibration yet

## Steps

1. Choose one bounded comparison surface for resampler quality, such as an
   alias-energy metric or a frozen artifact comparison report.
2. Implement that proof surface in `signal-dsp-resample`.
3. Add focused tests or benchmarks that record low-quality versus
   `BandLimited` behavior explicitly.
4. Rerun the focused resampler and dependent analysis validation surface plus
   repo health.

## Acceptance Criteria

- the resampler quality tiers have explicit comparative evidence, not just enum
  labels
- the proof surface is stable enough for future strict-lane regression checks
- focused validation passes

## Evidence Required

- batch log for the next `g09.009` tranche
- validation actually run
- explicit note that semantic calibration remains deferred until resampler
  proof posture is complete

## Stop Conditions

- the batch starts redesigning the semantic-tagging stack instead of proving
  resampler posture
- the comparison surface turns into a large benchmark platform instead of one
  bounded proof seam

## Next Task

Implement this resampler proof batch, then reassess whether `g09.009` should
stay in resampler fidelity evidence or switch to the semantic calibration seam.
