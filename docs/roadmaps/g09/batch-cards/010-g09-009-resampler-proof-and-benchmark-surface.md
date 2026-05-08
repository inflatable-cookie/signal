# 010 - g09.009 Resampler Proof And Benchmark Surface

Status: complete
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

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/011-g09-009-semantic-calibration-baseline.md`.

## Outcome

`signal-dsp-resample` now has a stable machine-readable comparison surface for
its quality tiers. The crate exposes `ResampleQualityComparisonReport` and
artifact metrics that compare `Nearest`, `Linear`, and `BandLimited` output
against one frozen input/rate conversion pair, and the focused proof tests show
that the `BandLimited` mode materially reduces alias-prone output while
preserving deterministic chunked/offline behavior.

## Validation Run

- `cargo test -p signal-dsp-resample`
- `cargo check -p signal-analysis-embed`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`
