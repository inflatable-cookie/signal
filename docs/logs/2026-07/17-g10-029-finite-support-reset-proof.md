# g10.029 Finite-Support Reset Proof

Date: 2026-07-17
Batch: 29.7V
Status: complete; reset law rejected

## Change

Added one report-only candidate beside the frozen 29.7T control. It reuses the
same representation, schedule, peak map, regions, owner, trajectory advance,
common rotation, inverse transform, and overlap. One condition differs:

- an analysis window crossing the known input boundary resets every active
  region to current analysis phase and creates no trajectory
- the first fully supported frame resets once because no boundary trajectory
  seeds it

No threshold, detector, local-time change, unlock, blend, window, picker,
region, owner, scale, classifier, mid/side, or post-render repair was added.

## Result

- calibrated failures: `1/48` frozen, `4/48` candidate
- row-complete improvements: `30/48` frozen, `29/48` candidate
- rows with a metric regression: `18/48` frozen, `19/48` candidate
- local failures: `11/48` frozen, `19/48` candidate
- previously passing local rows newly failing: `9/37`
- original local failures fixed: `1/11`
- calibrated candidate misses: all four short `0.75x` image rows
- dedicated structure, identity, swap, polarity, gain, silence, and repeat:
  pass
- candidate parity errors against the frozen mono control: `1.262698`,
  `1.262698`, `0`, `3.82e-14`,
  `5.050797`; fail
- six-row mono corpus: zero hard failures and zero row-complete regressions

Candidate states are `46,490` tracked, `13,307` reset, `165` silent, `59,797`
regions, and `5,946` owner switches. The reset law changes state ownership as
intended; the rejection is not a dormant branch.

## Original Failure Boundary Map

Each cell is normalized-Gram residual for frozen 29.7T / finite-support reset /
Rubber Band. `H` and `T` are the first and last of eight local windows.

| Ratio | Frames | Phase | Aligned | H | T |
| --- | ---: | ---: | --- | --- | --- |
| 2.00 | 8000 | 0.00 | yes | 0.016735 / 0.021335 / 0.004679 | 0.017833 / 0.017723 / 0.001882 |
| 2.00 | 8000 | 0.00 | no | 0.015551 / 0.020415 / 0.002628 | 0.016564 / 0.019681 / 0.002203 |
| 1.50 | 8000 | 0.37 | yes | 0.011971 / 0.013772 / 0.001322 | 0.008458 / 0.010083 / 0.001523 |
| 2.00 | 8000 | 0.37 | yes | 0.018248 / 0.016529 / 0.002217 | 0.013539 / 0.023547 / 0.002105 |
| 0.75 | 16384 | 0.00 | yes | 0.003371 / 0.001382 / 0.000348 | 0.007216 / 0.005855 / 0.000937 |
| 2.00 | 16384 | 0.00 | yes | 0.008739 / 0.007378 / 0.000745 | 0.007656 / 0.004610 / 0.000192 |
| 2.00 | 16384 | 0.00 | no | 0.007915 / 0.008964 / 0.002115 | 0.006812 / 0.009980 / 0.003377 |
| 0.75 | 16384 | 0.37 | yes | 0.003924 / 0.002157 / 0.000538 | 0.008453 / 0.006920 / 0.000667 |
| 2.00 | 16384 | 0.37 | yes | 0.009056 / 0.005859 / 0.000258 | 0.008698 / 0.004714 / 0.000232 |
| 0.75 | 16384 | 0.37 | no | 0.003704 / 0.003743 / 0.001060 | 0.006819 / 0.004745 / 0.002123 |
| 2.00 | 16384 | 0.37 | no | 0.008465 / 0.005987 / 0.003706 | 0.007716 / 0.008751 / 0.003180 |

Reset improves some boundary windows and worsens others. It does not reproduce
Rubber Band's consistently lower boundary residual. The result is phase- and
material-dependent despite an exact trigger.

## New Local Regressions

The nine newly failing rows are:

- tone: short aligned `0.75x`, short off-bin `1.5x` at both phases, long off-bin
  `0.75x`, and long off-bin `1.5x`
- image: every short `2.0x` row

The only original failure fixed is the long, phase-zero, aligned `2.0x` tone.
Blanket reset therefore trades rows instead of closing the boundary.

## Evidence

- frozen stereo: `eff52febad8c0fb8`
- candidate stereo: `226737df336507e9`
- candidate mechanics: `205981c0d2a99a21`
- candidate mono corpus: `81029247d139e4fa`

All repeat exactly.

## Decision

Reject `FiniteSupportReset`. Do not tune its range or split head from tail.
The proof shows that nonstationary boundary ownership matters, but an
unconditional reset is not the missing complete law.

The next step is architecture review, not another renderer. The direct split
matches the source-backed distinction between ordinary, locked, reset, and
unlocked/material-guided states. Batch 29.7W must decide whether a bounded
complete material-state kernel is justified or this family closes.
