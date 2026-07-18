# g10.029 Normalized Material Objective Gate

Date: 2026-07-18
Batch: 29.7AO
Status: rejected at stereo gate

## Result

The normalized sliced renderer now runs the frozen Rule 31V material policy.
Each channel computes ratio-aware ordinary recurrence first. One global
classifier then selects reset, attack, unlocked ordinary, or tracked lock.
Compatible lock may borrow the greatest-energy trajectory below `6000 Hz`;
each source/output layer retains its own magnitude and current analysis-
relative phase before per-channel inverse synthesis.

Synthetic structure and mechanics pass. Reset, attack, unlock, and lock all
execute. Duplicate, mono-parity, silent-peer, and swap errors are zero.
Source/output slab high-water is `5/2`; the guidance dependency is exactly
`19` frames. Structural and nonfinite failures are zero. The report repeats at
hash `0edf7cc256282813`.

The corrected stereo stage rejects. Across `48` repeated rows:

- calibrated failures: `46/48`
- improved local windows: `110/384`
- Signal-relative local-row failures: `44/48`
- maximum normalized-Gram residual: `0.86973539821584`
- structural failures: `0`
- evidence hash: `ff4603accdb456e6`

This is far outside the Rule 31V comparator envelope. Failure spans tone and
image controls at every ratio; only part of the `1.5x` image group avoids the
local-row failure. The result is deterministic and structurally valid, so the
next question is coefficient-to-waveform stereo ownership, not capacity or
randomness.

## Stop

The six-row mono and long-development stage did not run. No policy change,
retry, row repair, audio export, listening, or holdout access occurred.

## Validation

- normalized material synthetic gate: pass
- normalized material corrected stereo gate: rejected as recorded above
- Rule 31T normalized sliced regression: pass, hash `0407f765c7d84375`
- Rule 31U guided linked-phase regression: pass, hash `90c10cd2e66d4faf`
- Rule 31R fixed Stage A regressions: pass, hash `79b0cc2047f563b6`
- `cargo test -p signal-dsp-stretch`: pass
- release missing-docs check: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass

`cargo fmt --all --check` remains blocked by pre-existing formatting drift in
`signal-dsp/src/binaural.rs`, `signal-render-plane/src/binaural_bank.rs`, and
`signal-render-plane/src/convolution_reverb.rs`. Those unrelated files were
not changed.

## Next Task

Run Batch 29.7AP under Rule 31W. Trace the first relation divergence through
source layers, ordinary/state ownership, output-layer projection, inverse
slices, and outer overlap. Compare ownership order with pinned source records
before authorizing another candidate.
