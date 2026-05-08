# 009 - g09.009 Resampler Quality-Tier Foundation

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.009
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/009-dsp-fidelity-and-semantic-analysis-realism-uplift.md
Auto-start next card: no

## Objective

Start `g09.009` with the most bounded fidelity seam: turn
`signal-dsp-resample` from an interpolation-only surface into an explicit
quality-tier substrate with one genuinely higher-quality mode that performs
band-limited low-pass smoothing instead of only swapping interpolation choice.

## Scope

- harden `crates/signal-dsp-resample/src/lib.rs`
- keep the existing deterministic fast modes explicit and available
- add one higher-quality mode with clear low-pass or band-limited behavior
- add focused tests that show the new mode changes high-frequency downsampling
  behavior materially
- keep the batch inside `signal-dsp-resample`; do not widen into semantic
  calibration yet

## Steps

1. Define explicit resampler quality tiers and their posture.
2. Implement one higher-quality mode with proper low-pass smoothing for
   downsampling while preserving streaming determinism.
3. Keep the existing nearest and linear paths explicit as lower-quality modes.
4. Add focused fidelity tests that distinguish the high-quality mode from the
   existing interpolation-only modes.
5. Rerun the focused resampler validation surface plus repo health.

## Acceptance Criteria

- resampling quality posture is explicit rather than implied by interpolation
  choice alone
- one higher-quality mode performs real low-pass smoothing or band-limited
  behavior
- focused tests prove the high-quality mode differs materially from the old
  interpolation-only path
- focused validation passes

## Evidence Required

- batch log for the first `g09.009` tranche
- validation actually run
- explicit note if semantic-calibration work remains intentionally deferred

## Stop Conditions

- the batch starts redesigning the wider semantic-tagging pipeline instead of
  staying inside resampler quality posture
- the change requires a broader realtime-policy or host-facing contract not
  already captured in contract `077`
- the work drifts into benchmarks or demo substrate beyond what the batch needs
  to prove the new tier

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`.

## Outcome

`signal-dsp-resample` now exposes explicit quality tiers instead of only
interpolation-choice posture. The crate keeps `Nearest` and `Linear` as fast,
deterministic low-quality modes and adds a `BandLimited` windowed-sinc path
that performs real low-pass smoothing during downsampling. Focused tests prove
that the new mode is streaming-safe, deterministic across chunk boundaries, and
materially attenuates alias-prone content compared with linear interpolation.

## Validation Run

- `cargo test -p signal-dsp-resample`
- `cargo check -p signal-analysis-embed`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`
