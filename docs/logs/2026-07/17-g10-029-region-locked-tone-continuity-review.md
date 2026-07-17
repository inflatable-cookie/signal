# g10.029 Region-Locked Tone-Continuity Review

Date: 2026-07-17
Batch: 29.7U
Status: complete; one boundary-reset proof authorized

## Frozen Evidence

The 29.7T renderer, `48` rows, thresholds, mono result, and hashes remain
unchanged:

- stereo: `eff52febad8c0fb8`
- mechanics: `ad907a31d6ae940a`
- mono corpus: `c062525dfa1da3ff`

No renderer, peak, region, trajectory, reset, window, scale, threshold, or
blend changed.

## Failure Map

`H` and `T` identify the first and last of eight equal local windows. IPD is
whole-render radians. Residual is normalized-Gram error in the worst boundary
window.

| Ratio | Frames | Phase | Aligned | Signal whole IPD | Candidate whole IPD | Candidate interior IPD | Rubber Band whole IPD | Boundary | Candidate residual | Rubber Band residual | Reset regions | Owner switches |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 2.00 | 8000 | 0.00 | yes | 0.005235 | 0.004125 | 0.000002083 | 0.000115 | T | 0.017833 | 0.001882 | 108 | 1589 |
| 2.00 | 8000 | 0.00 | no | 0.014748 | 0.009708 | 0.000000636 | 0.000362 | T | 0.016564 | 0.002203 | 120 | 2081 |
| 1.50 | 8000 | 0.37 | yes | 0.002541 | 0.002189 | 0.000000177 | 0.000346 | H | 0.011971 | 0.001322 | 100 | 603 |
| 2.00 | 8000 | 0.37 | yes | 0.005130 | 0.003335 | 0.000000275 | 0.000277 | H | 0.018248 | 0.002217 | 126 | 1542 |
| 0.75 | 16384 | 0.00 | yes | 0.001032 | 0.001397 | 0.000000163 | 0.000252 | T | 0.007216 | 0.000937 | 32 | 1109 |
| 2.00 | 16384 | 0.00 | yes | 0.004135 | 0.004741 | 0.000000060 | 0.000169 | H | 0.008739 | 0.000745 | 108 | 3249 |
| 2.00 | 16384 | 0.00 | no | 0.008441 | 0.004723 | 0.000001218 | 0.000019 | H | 0.007915 | 0.002115 | 120 | 4193 |
| 0.75 | 16384 | 0.37 | yes | 0.001783 | 0.000075 | 0.000000207 | 0.000109 | T | 0.008453 | 0.000667 | 33 | 1175 |
| 2.00 | 16384 | 0.37 | yes | 0.006834 | 0.001973 | 0.000000198 | 0.000006 | H | 0.009056 | 0.000258 | 126 | 3261 |
| 0.75 | 16384 | 0.37 | no | 0.000843 | 0.000505 | 0.000000119 | 0.000281 | T | 0.006819 | 0.002123 | 39 | 609 |
| 2.00 | 16384 | 0.37 | no | 0.002419 | 0.003355 | 0.000000086 | 0.000218 | H | 0.008465 | 0.003706 | 107 | 4297 |

Seven failures are `2.0x`, three are `0.75x`, and one is `1.5x`. Length,
phase, and bin alignment do not separate the set. Every row has monotonic
fixed-ratio source centres and therefore zero trajectory breaks. Resets and
owner switches are present, but the same rows have candidate interior IPD from
`5.97e-8` to `2.08e-6` radians. They do not explain the boundary-only loss.

All eleven maximum candidate residuals occur in window `H` or `T`. Windows
`1` through `6` remain stable. Rubber Band is lower in the same worst boundary
window on every row.

## First Divergence

Peak integration, predecessor assignment, and common rotation remain coherent
inside fully supported steady frames. One common complex rotation cannot alter
a frame-local interchannel coefficient relation. The candidate's near-zero
interior IPD confirms that result after synthesis.

At finite support edges, reflected-support analysis frames are not stationary
tone frames. Multiple boundary-conditioned frames carry different region
rotations. Each frame preserves its own relation, but their overlap sum does
not preserve the finite input's normalized Gram relation. The first measured
divergence is therefore overlap of boundary-conditioned tracked frames, not
normalization or
a general overlap defect. Exact per-channel normalization, mechanics, and the
image-row improvement remain intact.

Image controls contain several persistent partials with stable peak regions.
The common-region rotation removes their prior field-wide stereo conflict.
Steady tones need little interior repair; their remaining broadband structure
is created only by finite-support onset and offset. Tracking that structure as
stationary is the missing state.

## Decision

Authorize one parameter-free `FiniteSupportReset` proof. When an analysis
window intersects samples outside the known input domain, every active region
uses current analysis phase and creates no trajectory. The first fully
supported frame then resets once before normal predecessor tracking resumes.

This is a Signal-owned deterministic boundary condition, not an inferred
threshold. It follows the source-backed phase-vocoder law that nonstationary
attacks require reset rather than stationary phase propagation. Rubber Band's
explicit reset states support the operator boundary but supply no expression,
range, constant, or detector.

Batch 29.7V may test only this law against the frozen 29.7T kernel. No other
rescue or product-facing work opens.
