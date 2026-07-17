# g10.029 Material-State Boundary Review

Date: 2026-07-17
Batch: 29.7W
Status: complete; shared rotation closed as a complete kernel

## Frozen Evidence

- 29.7T stereo `eff52febad8c0fb8`
- 29.7T mechanics `ad907a31d6ae940a`
- 29.7T mono corpus `c062525dfa1da3ff`
- 29.7V stereo `226737df336507e9`
- 29.7V mechanics `205981c0d2a99a21`
- 29.7V mono corpus `81029247d139e4fa`

No renderer, detector, classifier, state, scale, threshold, or comparator
changed.

## Failure Classification

The 29.7V candidate has 19 local failures: 15 tone and four image; five at
`0.75x`, four at `1.5x`, and ten at `2.0x`; 11 short and eight long. The four
image failures are every short `2.0x` row. Its separate four calibrated misses
are every short `0.75x` image row, which retain local consistency but fail the
whole/interior image gate.

Nine new local failures comprise five tone rows and the four short `2.0x`
image rows. One original long, phase-zero, aligned `2.0x` tone clears. Of the
ten retained original failures, five peak at the head and five at the tail.
Reset helps and harms both sides; no head/tail split exists.

The direct split is material-state evidence. Tracking every boundary protects
stable image relations but treats reflected tone boundaries as stationary.
Resetting every boundary avoids some tone trajectories but destroys stable
image and mono continuity, especially when boundary frames occupy much of a
short render.

## Source Comparison

Pinned Rubber Band R3 computes ordinary advance before material guidance. It
then chooses reset, unlocked, or peak-locked output by frequency. Channel
borrowing is conditional inside the locked branch, and frequency scale
ownership is independent of phase-state ownership.

Bungee independently supports common complete-region rotation but not the full
material state set. Signalsmith supports ordinary advance and channel-relative
ownership but has no complete transient model. Papers independently support
peak lock, transient reset, and multichannel owner selection. Only Rubber Band
in the current record composes classifier guidance, explicit unlock, and
simultaneous nonoverlapping frequency-owned scales.

## Decision

Close shared rotation as a universal renderer. Keep it as evidence for the
harmonic/locked state of a possible future complete kernel. Do not authorize
another reset, boundary, peak, scale, or blend experiment.

A clean-room complete material-state renderer is not ready. Its two unresolved
seams are material-guided ordinary/unlocked ownership and nonoverlapping
frequency-owned scale synthesis. Batch 29.7X must seek independent support for
both before implementation. Batch 29.8 stays closed.
