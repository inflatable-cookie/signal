# g10.029 Transient-Evidence Measurement Contract

Date: 2026-07-12
Status: frozen

## Detector

- square-root Hann window `2048`, hop `128`, FFT `4096`
- centered wrapped mixed phase difference normalized to ideal `0/1`
- positive-frequency cells `1..2046`
- per-channel energy floor: frame energy divided by `4096^2`
- percussive when closer to ideal impulse `1` than sinusoid `0`
- linked percussive-magnitude occupancy across channels
- no smoothing
- peak: occupancy at least `0.5`, strict rise, non-strict fall

## Proof Boundary

Measure the unchanged synthetic controls, stereo variants, gain/polarity
invariance, perturbation, dense events, false positives, finite values, closure,
and deterministic hashes. Produce no schedule or audio. Failure returns to
operator review without a parameter sweep.

## Next Task

Run Batch 29.6AT detector measurement. Do not implement schedule mapping, phase,
or stretched synthesis.
