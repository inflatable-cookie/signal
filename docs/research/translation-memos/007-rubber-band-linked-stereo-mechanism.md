# Rubber Band Linked-Stereo Mechanism

Status: promoted
Date: 2026-07-16
Owner: dsp
Target: `g10.029` Batch 29.7L

## Question

Why does Rubber Band R3 preserve the calibrated stereo controls while Signal's
same-bin reference-relative recurrence retains a small tone and image residual?

## Provenance Boundary

The installed comparator is `/opt/homebrew/bin/rubberband` `4.0.0`. Homebrew's
formula points to the official `rubberband-4.0.0.tar.bz2` archive with SHA-256
`af050313ee63bc18b35b2e064e5dce05b276aaf6d1aa2b8a82ced1fe2f8028e9`.
The archive identifies Mercurial node `1ea2558505589ca7dab9fd6dd6facd97cd119aaf`.
Its source matches the official Git mirror tag `v4.0.0`, commit
`1d95888bec3ae0a17c0c4af791810d5a63f6bc35`, apart from repository-only
packaging material.

The executable was poured from Homebrew's `arm64_tahoe` bottle with SHA-256
`65ba1e050a7368f043369932bfc493de4f33f9dda9f6863250960559b5bdf1d8`.
The inspected formula tap head is
`2c61c45c3247d7c69f69adf88761d38072072a52`; no build options were used.

Rubber Band is GPL-2.0-or-later with a separate commercial licence option.
Signal uses the source only as architecture evidence. No implementation
expression, constant, lookup table, or threshold transfers into Signal.

## Verified Mechanism

R3 analyses each channel separately, then advances phase for all channels in
one synchronized operation. The default stereo mode is not independent stereo:

- each channel tracks current and previous spectral peaks
- each bin records the channel with greatest current magnitude
- a peer may borrow that channel's peak trajectory only inside the active
  channel-link range
- borrowing requires compatible previous peak ownership; otherwise the channel
  retains its own trajectory
- the peer keeps a current analysis-relative phase offset from the borrowed
  peak rather than taking a bare dominant-channel phase
- the linked range is bounded and becomes narrower for long stretches
- reset, unlocked, kick, and peak-locked states remain distinct

The public `--centre-focus` mode is a separate stronger policy. For two
channels it converts left/right to mid/side before analysis, widens channel
coupling, synchronizes phase processing, prevents side silence from repeatedly
forcing centre resets, then converts back after synthesis.

## Behavioral Differential

One repeat-stable report renders the frozen 29.7J tone and correlated-image
matrix through R3 standard and R3 centre-focus. It covers two source lengths,
two starting phases, aligned and off-bin tones, and `0.75x`, `1.5x`, and `2.0x`.
There are `48` rows per mode and every paired output hash differs.

| Mode | Calibrated failures | Maximum tone IPD | Maximum image mid/side | Maximum image correlation | Maximum image relation residual |
| --- | ---: | ---: | ---: | ---: | ---: |
| R3 standard | `0/48` | `0.001339 rad` | `0.028627 dB` | `0.000578` | `0.001462` |
| R3 centre-focus | `4/48` | `0.001179 rad` | `0.026705 dB` | `0.002044` | `0.003142` |

Centre-focus changes all `48` renders but breaches the calibrated image
correlation or relation-residual ceiling on four `2.0x` rows. It is therefore
an active stereo mechanism with a measurable tradeoff, not a universal quality
upgrade. The standard mode remains the comparator Signal must explain.

## First Architectural Difference

Signal selects one reference independently at every same-frequency bin and
projects the peer's current same-bin relation on every active coefficient.
Rubber Band standard instead couples channels at tracked peak-trajectory level:
eligibility is conditional on compatible peak history and limited by frequency
and stretch state. Same-bin energy chooses a possible trajectory owner, but it
does not make every peer coefficient globally linked.

This is the first verified difference upstream of synthesis. It also explains
why the rejected render-wide matrix was too late and why centre-focus can trade
individual-channel fidelity for stronger image focus.

## Promoted Invariant

Cross-channel phase ownership belongs to tracked peak regions, not isolated
same-frequency coefficients or a completed render. Sharing must be conditional
and frequency-bounded. Inside an eligible region, preserve the peer's local
analysis-relative phase and magnitude; outside it, retain channel-owned phase
evolution.

This is a Signal-owned architectural invariant. Rubber Band's peak picker,
frequency limits, offset scaling, reset logic, and constants remain excluded.
Mid/side is not promoted.

## Next Proof

Define one Signal-owned report-only peak-region trajectory candidate. Freeze
peak identity, cross-channel eligibility, and frequency ownership before
rendering. Compare it with current reference-relative recurrence using the
unchanged mechanics and calibrated gates. No threshold sweep, centre-focus
clone, listening, dynamic ratio, realtime, routing, or production change is
open.

## Signal Feasibility Result

Batch 29.7M tests one independent realization: local-maxima peak identity,
nearest-peak frequency regions, and exact agreement on the previous peak owner.
It is active, repeat-stable, and exact on stereo mechanics, but raises
calibrated failures from `20/48` to `29/48`, regresses `35/48` rows, and loses
local consistency on `32/48`. Evidence `31a8b2eaae086fc8` rejects the
candidate without tuning.

The promoted peak-region invariant survives, but it is incomplete alone. The
next proof must triangulate material-state ownership and ordering rather than
altering peak thresholds, compatibility, or frequency bounds.

## Sources

- [Rubber Band v4.0.0 source tag](https://github.com/breakfastquay/rubberband/tree/v4.0.0)
- [R3 phase advance at v4.0.0](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [R3 stretcher at v4.0.0](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/R3Stretcher.cpp)
- [R3 guide at v4.0.0](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/Guide.h)
- [Rubber Band stretcher API at v4.0.0](https://github.com/breakfastquay/rubberband/blob/v4.0.0/rubberband/RubberBandStretcher.h)
- [Official Rubber Band site](https://breakfastquay.com/rubberband/)
