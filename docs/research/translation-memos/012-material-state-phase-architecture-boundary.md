# Material-State Phase Architecture Boundary

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7W
Contract: `082`, Rule 31H

## Question

Decide whether the frozen 29.7T and rejected 29.7V evidence supports one
complete clean-room material-state renderer, or closes shared rotation as a
complete kernel family.

## Direct Evidence

| Class | Frozen 29.7T | Finite-support reset | Finding |
| --- | --- | --- | --- |
| calibrated stereo | `1/48` failures | `4/48` failures | blanket reset breaks all four short `0.75x` image controls |
| local consistency | `11/48` failures | `19/48` failures | reset adds nine failures and clears one |
| material | 11 tone failures | 15 tone, 4 image failures | one state law cannot serve both stable image and boundary tone structure |
| ratio | 3 at `0.75x`, 1 at `1.5x`, 7 at `2.0x` | 5 at `0.75x`, 4 at `1.5x`, 10 at `2.0x` | ratio does not isolate a reset range |
| length | 4 short, 7 long | 11 short, 8 long | short renders expose excessive reset ownership; long rows still fail |
| mono | exact parity | parity errors to `5.050797` samples | boundary reset changes the mono kernel, not only stereo relation |

Of the original failures retained by 29.7V, five peak at the head and five at
the tail. The one cleared original row peaked at the head. The reset improves
and worsens measured windows on both sides, so boundary side does not select a
law. The frozen report records only the maximum residual, not the maximum-
window index, for the nine new rows; no stronger side claim is justified.

The state transition is exact. Frozen 29.7T tracks reflected-support frames as
stationary regions. Batch 29.7V instead runs boundary reset, one first-
supported reset, then normal tracking at the head; the tail changes from normal
tracking to boundary reset. Stable image rows lose relation when reset owns too
much of a short render. Tone rows lose relation when reflected boundary
structure is tracked. Neither universal choice is correct.

## Pinned Source Order

Rubber Band R3 does not choose between universal track and universal reset. It:

1. computes ordinary instantaneous-frequency advance for every bin
2. derives per-channel harmonic, percussive, and residual guidance
3. assigns each frequency to one synthesis scale
4. selects reset or kick ranges first
5. leaves unity and unlocked residual ranges on ordinary advance
6. otherwise applies peak locking with a frequency- and ratio-dependent local
   analysis offset
7. borrows another channel's peak only inside both channel-link ranges and only
   when predecessor peak histories agree

Reset, unlock, peak lock, channel link, and scale ownership are separate
decisions. Common rotation is the identity-locking case inside the locked
branch, not the complete phase kernel.

## Independent-Support Matrix

| Seam | Independent support | Status |
| --- | --- | --- |
| ordinary horizontal advance | standard phase-vocoder literature; Signalsmith | supported |
| peak-region lock | Laroche-Dolson; AudioTSM; Bungee | supported |
| transient/discontinuity reset | Röbel; Bungee | supported |
| linked-channel peak ownership | Dorran-Lawlor-Coyle; Signalsmith | supported |
| explicit material-guided unlock | Rubber Band; Signalsmith only supplies a documented high-ratio diffusion hack | incomplete |
| simultaneous nonoverlapping frequency-owned scales | Rubber Band exact source only in the current record | incomplete |
| classifier-to-state-to-scale ordering | Rubber Band exact source only | incomplete |

Individual states have strong support. The composition that decides when each
state and scale owns a bin does not. Porting that composition from one GPL
specimen would cross the clean-room boundary; inventing it would restart the
parameter churn this review exists to stop.

## Decision

Close `SharedRotationRegionLocked` as a complete renderer family. Retain common
region rotation as validated evidence for a future harmonic/locked state. Do
not tune reset range, boundary side, peak map, scale, or blend, and do not
promote 29.7T or 29.7V.

Do not authorize a complete material-state renderer yet. First close the two
missing independent seams: material-guided ordinary/unlocked ownership and
nonoverlapping frequency-owned scale synthesis. Any later candidate must freeze
the complete state map and scale ownership before rendering, run once, and pass
the existing mono, stereo, boundary, and comparator gates without parameter
rescue.

## Sources

- [Rubber Band R3 guide](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/Guide.h)
- [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [Rubber Band R3 stretcher](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/R3Stretcher.cpp)
- [Signalsmith Stretch `1.3.2`](https://github.com/Signalsmith-Audio/signalsmith-stretch/tree/57b93f4e9206a089a45387eaa39bdc9f310d3308)
- [Bungee `2.4.24`](https://github.com/bungee-audio-stretch/bungee/tree/746833f68a574d997ec50443e7cfd2d37b026302)
- [Röbel transient phase-vocoder paper](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf)
- [Dorran, Lawlor, and Coyle multichannel TSM](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf)

## Next Task

Run Batch 29.7X as independent material-state kernel research. Implement
nothing. Require a second source or published basis for both missing seams
before another renderer card can exist.
