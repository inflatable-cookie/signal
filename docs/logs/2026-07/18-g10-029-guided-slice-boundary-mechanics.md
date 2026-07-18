# g10.029 Guided Slice-Boundary Mechanics

Date: 2026-07-18
Batch: 29.7AN
Status: passes

## Result

The Rule 31R synchronized channel-state workspace now accepts explicit sample
rate and common hop without changing the passing fixed-frame result. One
workspace advances once per normalized global tick. Its phase decision then
feeds both active slice layers while each layer retains its own magnitude and
current analysis-relative phase. Slice lifetime does not own state lifetime.

The three proof rates use the frozen Rule 31T geometry and representation hash
`0407f765c7d84375`. Across `3/6/14` slices, update counts are exactly
`64/112/240`; dual-layer counts are `32/80/208`. Reset, attack, ordinary,
unlocked, and compatible locked branches all execute in interior and
immediately before, at, and after slice-boundary contexts.

Duplicate, mono parity, silent peer, and swap coefficient errors are zero.
Maximum layer-magnitude error is `1.1102230246251565e-16`; maximum retained
analysis-relative-phase error is `4.440892098500626e-16`. Region high-water is
`32/100/107` against `191/592/631` capacity. The longest atom-visit rows are
`45840/142080/151440`; region visits are `7680/24000/25680`. Continuity,
capacity, update, finite, layer, overflow, and repeat failures are zero. A third
layer is rejected before state advances. Evidence hash: `90c10cd2e66d4faf`.

## Boundary

This proves state mechanics, not material classification or sound quality.
Guidance is scripted only to cover state branches. No classifier, threshold
tuning, stretched quality audio, objective row, listening artifact, or holdout
access exists.

Batch 29.7AO remains closed. Batch 29.7ANR must first preregister or reject the
unchanged Rule 31R material policy and complete objective evidence matrix on
the normalized geometry.

## Validation

- focused Rule 31U release proof: `2` passed
- Rule 31R guided-kernel regression: `3` passed
- Rule 31T normalized-frame regression: `2` passed
- `signal-dsp-stretch` debug suite: `269` tests passed across library, binary,
  integration, and documentation targets
- release missing-docs check passed
- strict clippy still reports the existing crate warning backlog; no finding
  points at the Rule 31U files
- `effigy doctor` remains at the existing god-file and attention-marker
  baseline; the Rule 31U modules add no finding

## Next Task

Run Batch 29.7ANR. Freeze or reject every Rule 31R policy term, the complete
objective evidence matrix, fixed work/capacity, failure ordering, and
no-sweep/no-row-repair rule on the normalized lattice. Keep implementation,
quality audio, objective execution, listening, holdout, Batch 29.7AO, Batch
29.8, and product work closed.
