# Linked-Stereo State And Trajectory Policy

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7N
Contract: `082`, Rule 31H

## Problem

Batch 29.7M rejected one Signal-owned peak-region renderer. Its closeout
assigned the loss to missing material-state policy. That conclusion was not
yet isolated. The frozen calibration matrix contains only steady tone and
correlated-image controls, while the candidate changed both cross-channel
sharing and the recurrence used outside shared regions.

29.7N separates those variables and triangulates the remaining state and
trajectory rules without transferring implementation expression or constants.

## Frozen Attribution

One repeat-stable ablation compares three renderers on the unchanged `48`-row
calibrated matrix:

| Renderer | Calibrated failures | Complete improvements from prior | Regressions from prior | Local regressions |
| --- | ---: | ---: | ---: | ---: |
| current reference-relative Signal | `20/48` | - | - | - |
| channel-independent recurrence | `40/48` | `8/48` | `40/48` | `34/48` |
| independent plus 29.7M shared regions | `29/48` | `26/48` | `22/48` | `4/48` |

The peak-region stage shares `546801` bins and leaves `1304591` independent.
Evidence `d2de8ca4df6330f6` repeats exactly with zero structural failures.

The split is material-specific. Shared regions improve all `24/24` tone rows
over independent recurrence and regress none. They improve only `2/24` image
rows and regress `22/24`. The candidate is therefore not rejected because peak
sharing is inactive or universally harmful. The dominant loss is the
channel-independent default, followed by an over-broad single-anchor law on
complex image material.

## Evidence Triangulation

### Stable trajectories

Laroche and Dolson distinguish identity locking from tracked, scaled locking.
Identity locking keeps current analysis-relative phase around a current peak.
Tracked locking instead matches the current peak to a predecessor and advances
from the predecessor's synthesis phase. Their evaluation reports consistently
better listening quality from tracked scaled locking, while also saying the
phase scale has weak theoretical grounding.

PhaVoRIT independently reports two failure modes relevant to 29.7M:

- constant-resolution peak picking produces shallow bass and MP3-like musical
  overtones
- unconstrained predecessor assignment blurs note onsets and causes
  high-frequency warble

Its remedies are frequency-dependent peak resolution and bounded trajectory
continuation. Those are separate variables and must not be folded into one
unmeasured Signal candidate.

The 2005 multichannel TSM paper and Signalsmith Stretch agree on one
cross-channel invariant: choose ownership at a common frequency location, then
preserve the original channel relationship. Rubber Band R3's default path also
keeps the requesting channel's peak location when it borrows a compatible peer
trajectory. Batch 29.7M instead used the dominant peer's own peak as one anchor
for both channels. That distinction explains its clean tone recovery and poor
multi-tone image result.

### Transients and reset

Röbel shows that attack bins violate stationary phase propagation and should be
reset near the transient's window centre. The treatment is bin-local, not a
whole-band replacement.

Ravelli, Sandler, and Bello show that stereo onset decisions must be coordinated
across locally related channels. Independent onset classification causes image
movement; shared classification restores coherence.

These findings define later reset ownership. They do not explain 29.7M because
its calibration controls contain no attacks.

### Broadband and unlocked material

Phase-gradient integration can preserve horizontal and vertical coherence for
broadband, non-sinusoidal material without explicit peak or transient states.
Signal's coherent predictor already owns that class. The 29.7N independent
ablation proves that a channel-independent `Unlocked` default is unsafe for
linked stereo. No separate kick law has enough independent evidence for
promotion.

## Signal Policy

Use three states with strict precedence:

1. `Reset`: a later, channel-coordinated and frequency-local attack override
2. `TrackedPeak`: a stable trajectory overlay with bounded predecessor identity
3. `Relational`: the current reference-relative coherent predictor, used by
   default and for every unclaimed bin

`Attack` is evidence that may trigger `Reset`, not a fourth phase law.
`Unlocked` and `Kick` are not promoted. Linked stereo must never fall back to
fully channel-independent recurrence merely because peak sharing is ineligible.

Within `TrackedPeak`, channel ownership remains frequency-aligned. Each channel
keeps its own region peak location. A compatible dominant peer may own the
trajectory evaluated at that location; it does not replace the location with
the peer's region peak. The trajectory advances from the matched predecessor's
synthesis state. Current-peak identity locking is not a tracked trajectory.

## Rejected And Deferred

- reject missing transient/reset state as the explanation for 29.7M
- reject channel-independent recurrence outside eligible regions
- reject one dominant peer peak as the anchor for both channel regions
- defer phase-offset scaling; published values are empirical and would create a
  parameter experiment
- defer frequency-dependent peak resolution and predecessor-distance policy
  until one reference-safe tracked identity overlay is isolated
- keep production, listening, dynamic ratio, realtime, routing, and promotion
  closed

## Next Proof

Batch 29.7O rejects the report-only reference-safe tracked-peak overlay. It
retains requesting-channel peak location, advances from matched predecessor
synthesis state with instantaneous-frequency unwrapping, and uses identity
analysis-relative offsets. It is active, repeat-stable, and mechanics-exact.
Failures rise from `20/48` to `25/48`; there are no row-complete improvements,
all `48` rows regress on at least one metric, and `34/48` lose local
consistency. Evidence is `ec1f63ad4bae9fc8`.

The result rejects a late tracked-peak overlay on Signal's completed
phase-gradient field. It does not reject peak ownership before or during phase
integration. Published peak-locked systems estimate peak phases first and use
them to derive phases for the surrounding region. Batch 29.7P attributes the
loss across anchors, interiors, and boundaries and promotes a complete
peak-owned eligible-region operation in translation memo 009.

## Sources

- [Laroche and Dolson, Improved Phase Vocoder Time-Scale Modification of Audio](https://doi.org/10.1109/89.759041)
- [Karrer, Lee, and Borchers, PhaVoRIT](https://quod.lib.umich.edu/i/icmc/bbp2372.2006.142/--phavorit-a-phase-vocoder-for-real-time-interactive-time?rgn=main;view=fulltext)
- [Dorran, Lawlor, and Coyle, Multi-Channel Audio Time-Scale Modification](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf)
- [Ravelli, Sandler, and Bello, Fast Implementation for Non-Linear Time-Scaling of Stereo Signals](https://dafx.de/paper-archive/2005/P_182.pdf)
- [Röbel, A New Approach to Transient Processing in the Phase Vocoder](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf)
- [Průša and Holighaus, Phase Vocoder Done Right](https://ltfat.org/notes/ltfatnote050.pdf)
- [Ottosen and Dörfler, A Phase Vocoder based on Nonstationary Gabor Frames](https://arxiv.org/abs/1612.05156)
- [Signalsmith Stretch design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/)
- [Rubber Band R3 phase advance, GPL architecture evidence only](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
