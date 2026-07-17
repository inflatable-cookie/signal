# Peak Owner And Phase-Field Order

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7P
Contract: `082`, Rule 31H

## Problem

Batch 29.7O applies tracked-peak identity locking after Signal has completed its
reference-relative phase field. The candidate is active and mechanics-exact,
but every calibrated row regresses somewhere. This batch determines whether
the failure is local to region seams, repairable by peak seeds, or caused by
mixed phase ownership across the whole eligible region.

## Frozen Attribution

The existing 29.7O renderer gained report-only tracing. Audio samples, peak
selection, eligibility, offsets, gates, and hashes remain unchanged. The trace
measures phase displacement from the relational field and stereo relation error
before and after the overlay for peak anchors, region interiors, and region
boundaries. It repeats over the same `48` frozen rows.

| Class | Changed coefficients | Relation bins | Before RMS | After RMS |
| --- | ---: | ---: | ---: | ---: |
| peak anchor | `200123` | `138894` | `0.057562` | `1.485181` |
| region interior | `527131` | `248946` | `0.038310` | `1.197048` |
| overlay boundary | `365548` | `158829` | `0.129766` | `1.182947` |

The result is not a seam effect. Anchors and interiors regress as strongly as
boundaries. Every ratio and both control families show the same direction. The
overlay displaces coefficient phase by `1.615` to `1.776` radians RMS and can
approach pi in every class. Evidence `e1713e619138301b` repeats exactly.

## Source Comparison

| System | Phase-owner order | Translation boundary |
| --- | --- | --- |
| peak-locked phase vocoder | estimate peak synthesis phase, then derive its region from analysis-relative phase | peak phase precedes region phase |
| nonstationary Gabor phase vocoder | estimate only peak channels; lock remaining channels to the owning peak | one owner constructs the complete region |
| RTPGHI | seed known prior-frame coefficients, then spread phase from the greatest-magnitude available coefficient through time and frequency neighbors | ownership is established during magnitude-ordered integration |
| Signalsmith Stretch 1.3.2 | make a horizontal prediction, combine upward and downward frequency predictions, choose the greatest-energy channel, then lock peer channels to that output | cross-channel locking is inside prediction, before synthesis |
| Rubber Band R3 | conditionally borrow a compatible tracked trajectory and derive local phase under that branch | architecture evidence only; expression and constants remain excluded |
| Signal 29.7O | complete relational integration independently, then overwrite eligible coefficients from separately evaluated channel anchors | rejected mixed-owner order |

These systems do not imply one shared algorithm. They do agree on the relevant
ordering invariant: the owner participates in construction of the phase field.
None supports replacing part of an already-complete field with independently
advanced channel anchors.

## Promoted Law

A tracked peak must not be a late overlay on a completed relational phase
field. Any later tracked-peak proof must construct each complete eligible
region under one phase-owner operation:

1. establish the tracked owner phase
2. derive every eligible region coefficient from that owner
3. preserve the peer's same-frequency current analysis relation inside the
   same operation
4. leave each ineligible region wholly owned by the current relational
   recurrence

Do not mix relational and tracked ownership inside one eligible region. A peak
seed that merely constrains a completed or independently integrated field is
not authorized.

## Rejected And Deferred

- reject boundary smoothing or a boundary-only repair
- reject peak seeds added after relational integration
- reject independently advanced channel anchors inside one linked region
- retain the frozen picker, compatibility, frequency ownership, offset scale,
  controls, and gates
- defer transient reset, listening, dynamic ratio, realtime, routing, and
  production use
- keep GPL implementation expression and constants excluded

## Next Proof

Batch 29.7Q may implement one report-only complete peak-owned region operator.
It must reuse the 29.7O picker, predecessor eligibility, tracked phase advance,
and identity offsets. The only changed variable is operator order and ownership:
one eligible region is constructed as one unit, including exact peer relation.
Current relational recurrence owns every ineligible region unchanged.

## Sources

- [Laroche and Dolson, Improved Phase Vocoder Time-Scale Modification of Audio](https://doi.org/10.1109/89.759041)
- [Ottosen and Dörfler, A Phase Vocoder based on Nonstationary Gabor Frames](https://arxiv.org/abs/1612.05156)
- [Průša and Søndergaard, Real-Time Spectrogram Inversion Using Phase Gradient Heap Integration](https://dafx.de/paper-archive/2016/dafxpapers/03-DAFx-16_paper_02-PN.pdf)
- [Průša and Holighaus, Phase Vocoder Done Right](https://ltfat.org/notes/ltfatnote050.pdf)
- [Signalsmith Stretch 1.3.2 pinned source](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h)
- [Signalsmith Stretch design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/)
- [Rubber Band R3 phase advance, GPL architecture evidence only](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
