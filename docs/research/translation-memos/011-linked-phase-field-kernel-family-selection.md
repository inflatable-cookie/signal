# Linked Phase-Field Kernel Family Selection

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7S
Contract: `082`, Rule 31H

## Question

Select at most one complete linked-stereo phase-field family before another
renderer. Compare joint multichannel phase-gradient integration with a
state-complete peak-locked phase vocoder. Do not reopen a local current-kernel
variant.

## Decision Matrix

| Boundary | Joint phase-gradient integration | Shared-rotation region-locked phase vocoder |
| --- | --- | --- |
| representation | uniform STFT; Gaussian-derived or measured phase gradients | invertible STFT; peak regions partition every active frame |
| horizontal owner | magnitude-ordered integration from prior-frame seeds | tracked peak trajectory from prior synthesis state |
| vertical owner | heap path integrates adjacent frequency gradients | one current analysis-relative rotation per complete peak region |
| transient policy | implicit; transient is stretched with no reset | complete-region reset when continuity fails; no separate detector in the first proof |
| stereo policy | no published joint integration law located | dominant channel advances peak; common rotation preserves every peer relation |
| weak/silent policy | thresholded bins receive arbitrary or analyzed phase depending realization | zero stays zero; unowned regions reset to current analysis phase; no random phase |
| synthesis | dual-window inverse STFT and overlap-add | inverse STFT and exact overlap-add on the same phase-owner grid |
| multiresolution | published filter-bank method requires uniform decimation; Signal's common-grid bank later failed synthesis guard | same region law can later run once per nonoverlapping frequency-owned scale |
| deterministic offline | bounded per-frame heap is feasible | bounded region state is feasible |
| realtime projection | RTPGHI demonstrates one-frame bounded integration | Bungee demonstrates dynamic multichannel grain processing |
| direct Signal evidence | fixed-grid kernel and exact-lattice repair already rejected | not yet tested as a complete replacement kernel |
| licensing/source boundary | papers are usable; public LTFAT implementation is GPL | papers plus MIT AudioTSM; Bungee MPL architecture only; Rubber Band not required |

## Phase-Gradient Finding

Průša and Holighaus report competitive mono listening at `1.5x` and `2x`
without peak picking or explicit transient detection. That is strong external
evidence. It does not erase Signal's direct result.

Batch 29.6F proved a bounded full phase-gradient kernel. Batch 29.6G then
rejected it on attack, timing, replica, formant, boundary, and combined gates.
Batch 29.6H corrected the rounded-hop lattice to at most `0.4` frame mapping
error and still passed only `3/60` combined rows. The family also has no
published multichannel heap law that jointly owns interchannel relation.

Adding a new stereo constraint would therefore place an unvalidated operator
on a mono family that already failed Signal's target. Joint phase-gradient
integration closes for the next renderer. Its research remains valid; its
rejection is specific to this lane and evidence set.

## Peak-Locked Finding

The alternative has independent support at every kernel seam:

- Laroche and Dolson define peak trajectories, regions of influence, and
  identity/scaled phase locking
- Dorran, Lawlor, and Coyle define multichannel greater-magnitude peak
  ownership while preserving the peer's original phase relation
- Röbel defines frequency-local reset when stationary phase propagation is
  invalid at attacks
- Ottosen and Dörfler define predecessor-region peak tracking, adaptive
  representation, region locking, and exact frame synthesis
- MIT AudioTSM provides a compact identity-locking implementation control
- MPL Bungee independently demonstrates a whole-kernel common-region rotation
  with dynamic multichannel grain processing

Rubber Band remains comparator and architecture evidence only. The selected
boundary does not need Rubber Band expression, constants, peak picker,
classifier, scale limits, or reset rules.

Signal's current production prototype is not this family. It phase-locks each
mono path independently, uses midpoint peak regions, advances bins before
locking, and obtains linked stereo by stretching mid and side separately. The
selected kernel instead builds one cancellation-safe native-channel peak map,
tracks predecessor regions, and applies one common region rotation across all
channels. It is a new report-only renderer, not a rename or promotion of the
prototype.

## Selected Clean-Room Kernel

Select `SharedRotationRegionLocked` as a separate report-only kernel family.
It does not call or wrap the current weighted predictor.

For each frame:

1. analyze every channel separately with the proven periodic-Kaiser,
   modified-half-bin representation and exact shared lattice
2. form joint energy as the per-bin maximum of channel energies; never sum
   channel complex coefficients for ownership
3. find joint peaks and divide the complete active spectrum at valleys
4. match each current peak to the prior region containing its frequency
5. choose the greatest-energy current channel at the peak with stable
   lower-channel tie breaking
6. advance the peak from that same channel's prior analysis phase, the
   trajectory's prior common rotation, and the actual adjacent centre interval
7. calculate one current peak rotation and apply it to every current analysis
   coefficient in every channel across the complete region
8. inverse-transform and overlap-add without a second phase owner

The common rotation preserves each channel's current vertical phase structure
and all current interchannel complex relations. Owner changes remain valid
because trajectory state retains the prior common rotation plus each channel's
prior analysis phase at the predecessor peak.

## Complete State Set

Every active region has one owner:

- `TrackedRegion`: viable predecessor; advance one peak and apply its common
  rotation to the complete current region
- `ResetRegion`: first frame, discontinuity, or unmatched predecessor; use
  current analysis phase for the complete region
- `Silent`: exact zero energy; emit exact zero and create no trajectory

`Relational`, weighted prediction, late overlay, independent peer recurrence,
random weak-bin phase, mid/side, and post-render image repair do not exist in
this kernel. A separate `Unlocked` state is not needed for complete ownership
and is closed in the first proof. No attack detector or local-time override is
active. This keeps the first renderer parameter-free at the material-policy
seam; explicit attack reset may reopen only after core-kernel attribution.

## Compatibility

- reuse Signal's coupled periodic-Kaiser/modified-half-bin representation,
  exact absolute analysis-centre scheduling, boundary support, deterministic
  tie rules, inverse transform, and overlap accounting
- the first proof stays fixed-grid; later multiresolution may assign each bin
  to one scale and run this complete region law inside that scale only
- all state is frame-bounded and preallocatable; realtime projection is
  structurally possible but remains closed
- current coherent mono and relational stereo remain unchanged controls
- dynamic ratio remains closed until fixed-ratio quality passes

## Bounded Proof

Batch 29.7T may implement exactly one report-only fixed-grid candidate at
`0.75x`, `1.5x`, and `2.0x`.

It must exercise all three states and compare current Signal, the candidate,
and Rubber Band on the unchanged mono integrity/corpus gates and `48`
calibrated stereo rows. Required mechanics include exact length, coverage,
finiteness, identity, silence, mono parity, hard pan, swap, polarity, scaled
duplicate, owner changes, trajectory breaks, and repeat hashes.

Passage requires zero calibrated stereo failures, zero local-consistency
failures, exact mechanics, and no row-complete mono regression against the
current coherent control. Stop after one candidate. Do not tune the peak map,
trajectory match, owner, reset threshold, region boundary, window, or blend.

## Sources

- [Průša and Holighaus, Phase Vocoder Done Right](https://ltfat.org/notes/ltfatnote050.pdf)
- [Průša and Søndergaard, Real-Time Spectrogram Inversion Using Phase Gradient Heap Integration](https://www.dafx.de/paper-archive/2016/dafxpapers/03-DAFx-16_paper_02-PN.pdf)
- [Laroche and Dolson, Improved Phase Vocoder Time-Scale Modification of Audio](https://doi.org/10.1109/89.759041)
- [Dorran, Lawlor, and Coyle, Multi-Channel Audio Time-Scale Modification](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf)
- [Röbel, A New Approach to Transient Processing in the Phase Vocoder](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf)
- [Ottosen and Dörfler, A Phase Vocoder Based on Nonstationary Gabor Frames](https://arxiv.org/abs/1612.05156)
- [AudioTSM pinned MIT implementation](https://github.com/Muges/audiotsm/blob/cf3875842bda44d81930c44b008937e72109ae9f/audiotsm/phasevocoder.py)
- [Bungee `2.4.24` pinned MPL architecture](https://github.com/bungee-audio-stretch/bungee/tree/746833f68a574d997ec50443e7cfd2d37b026302)
- [Rubber Band `4.0.0` GPL architecture evidence](https://github.com/breakfastquay/rubberband/tree/v4.0.0/src/finer)

## Next Task

Run Batch 29.7T as the one bounded fixed-grid
`SharedRotationRegionLocked` proof. Keep current production, Batch 29.8,
listening, dynamic ratio, realtime, routing, and cache identity closed.
