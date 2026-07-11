# g10.029 Three-Row Nyquist Completion Contract

Date: 2026-07-11
Status: decision frozen

## Input

Batch 29.6Y proved that retaining channel `1535` diagonal energy while removing
its cross-bin coupling reduces global condition from `2.9916436058` to
`1.1141796230`. Complete channel removal remains rejected at `2.6496906694`.

## Construction

Preserve raw channels `0..1534`, hop `384`, and the Rule 26 completion
magnitude `g(f)`. Replace the single completion row with three rows:

`H_r(f)=g(f)/sqrt(3) * exp(-i*2*pi*f*(128*r))`, for `r in {-1,0,1}`.

The delays are `-128`, `0`, and `+128` frames. Summed squared magnitude is
exactly `g(f)^2`. For same-residue bin separation `k/384`, the cross term is
proportional to the three cube roots of unity and vanishes for `k=1,2`.
Completion width `16*(0.5/1535)` is less than `3/384`, so no other nonzero
separation exists. Even integer delays make all three rows real at Nyquist.

This triplet is a Signal design inference. Compact frequency support, dense
sampling, measured frame bounds, and dual reconstruction follow the governing
nonstationary-Gabor proof boundary; the sources do not prescribe this triplet.

## Gate

Batch 29.6AA implements only the candidate construction and release matrix
proof at FFT length `4224`. It requires exact preserved-channel hashes,
analytic closure `1e-12`, all `11` Jacobi solves, global condition at most
`1.25`, the existing numerical gates, stable evidence hashes, and exact repeat.

Failure rejects the triplet and returns to geometry research. Passage opens a
separate identity reconstruction proof only, using the unchanged Rule 26
controls and error gates before any representative guard.

## Boundary

Do not run identity reconstruction, dual guards, phase, audio synthesis, corpus
rendering, linked stereo, dynamic ratio, cache, or product routing.

## Next Task

Implement Batch 29.6AA three-row Nyquist-completion matrix proof and stop after
its conditioning decision.
