# g10.031 Audited Renewal Stereo Rejection

Date: 2026-07-20
Status: candidate rejected at linked-stereo image preservation
Scope: Batch 31.25 closeout and isolated-candidate cleanup

## Result

Checkpoint `97ee70569bc2a9dd574970eefb19799873875946` passed compile,
construction `1/1`, structural `13/13`, and synthetic `9/9` without repair or
rerun. Concealed mono listening then passed as `15/15` ties against
PaulXStretch. The operator found no meaningful core-quality distinction and no
unusable or forbidden-character row.

The reported fade difference was real but not a pack artifact. Assembly added
no fade. The candidate applies a fixed `16384`-frame sine exterior envelope at
`44.1 kHz`, about `371.5 ms`.

## Stereo Stop

The retained full-mix stereo source corresponding to mono row `M005` measured
`-0.4516 dB` right-minus-left. Candidate `8x` renders measured:

- `space=0`: `+4.2147 dB`
- `space=0.5`: `+3.3660 dB`
- `space=1`: `+1.9453 dB`

The operator heard the rightward imbalance on speakers. Active-region
measurements confirmed that it persisted through the render. Below `250 Hz`,
all three `space` values retained `+1.5426 dB` rightward bias.

The source mid/side relationship was discarded. Whole-component orientation
then came from the first exactly non-zero mid and side samples, `149.39 dB`
and `141.04 dB` below their component peaks. Those incidental signs forced
renewed mid and side into render-wide anti-correlation. This is the dominant
cause.

## Decision

Reject the complete candidate at linked-stereo image preservation. Do not
repair or rerun checkpoint `97ee7056`. Independent stereo review cannot promote
a checkpoint with an already demonstrated source-image fault.

Retain the mono result as evidence for renewal synthesis, mapping, frame
blending, variance compensation, boundaries, and the PaulX-like target. It
does not validate the rejected first-sample stereo law.

The disposable worktree, branch, candidate source, tests, build state, and
listening assembly were deleted. No candidate DSP, public API, report, fixture,
cache, route, Loophole, or Chorus surface entered `main`.

## Next Task

Run Batch 31.26 only. Freeze one complete source-relative stereo renewal
successor brief without candidate DSP. Do not implement it in the same batch.
