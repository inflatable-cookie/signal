# g10.029 Time-Adaptive Transform Research

Date: 2026-07-11
Status: direction frozen

## Operator Direction

Continue successor research. Do not accept the dense painless candidate and do
not relax its real-spectrum or localization gates.

## Evidence

Three primary lines converge on time-adaptive Gabor resolution:

- [Liuni et al.](https://arxiv.org/abs/1109.6313) define variable-time Gabor
  resolution with dual-frame reconstruction.
- [Rudoy, Basu, and Wolfe](https://arxiv.org/abs/0906.5202) prove stable compact
  superposition frames with fast overlap-add reconstruction.
- [Akaishi, Holighaus, and Yatabe](https://arxiv.org/abs/2602.16421) identify
  percussion magnitude/phase mismatch and show short NSDGT windows can reduce
  smearing while long windows remain elsewhere.

The 2026 method's HPSS detector and stretching policy are not adopted. Signal
first proves the transform on declared schedules.

## Decision

Batch 29.6AI uses one `4096`-bin painless NSDGT with compact square-root Hann
windows at `512`, `1024`, `2048`, and `4096` frames. It proves schedule
transitions, diagonal dual reconstruction, support, real output, exact length,
and repeat only. No detector, phase modification, or stretched audio enters.

## Next Task

Implement Batch 29.6AI and stop at its identity decision.
