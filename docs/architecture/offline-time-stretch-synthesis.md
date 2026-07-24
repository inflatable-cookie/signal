# Offline Time-Stretch Synthesis

Status: active frozen baseline; successor program closed
Owner: dsp
Updated: 2026-07-24
Contract refs: `046`, `084`, separate creative path `085`; historical evidence `082`
Roadmap refs: transparent closeout `g10.030`; separate creative path `g10.031`

## Current Architecture

OfflineHighQuality uses a centered, padded `2048`-sample Hann STFT with a
`512`-sample analysis hop. It tracks spectral peaks, propagates phase with
identity locking, and resets phase on qualified expansion transients. The
render is normalized overlap-add, cropped to the exact sample-domain target,
and deterministic.

Compression and expansion may select the retained `1024/256` short-window path
through explicit promoted selectors. Identity remains a passthrough. Linked
stereo uses one mid/side transport surface. Pitch composition, stepwise dynamic
ratios, offline artifacts, cache identity, and RealtimePreview remain separate
bounded contracts.

This baseline is competitive but not the final professional-quality target.

## Retained Evidence Surface

The production branch keeps only:

- byte-exact and structural renderer tests
- exact length, finiteness, boundary, determinism, and allocation checks
- transient timing and crest measurement
- tonal texture and formant-boundary measurement
- full-render integrity, CPU, and peak-heap measurement
- Signal-versus-external rendered-output comparison
- level-matched blind listening across percussion, bass, vocals, sustains, and
  full mixes at compression and expansion ratios

Objective metrics diagnose and reject. Long-form listening decides promotion.

## Known Quality Gap

Operator evidence places Signal close to Rubber Band overall. The remaining
gap is concentrated in long expansion:

- slightly lower apparent resolution or greater grain
- subtle atonal ringing
- occasional softer transient definition
- occasional transient crest spikes
- possible small event-placement instability

The next renderer must improve these together. A tonal win that blurs attacks,
a transient win that produces ringing, or a mono win that breaks stereo is not
progress.

## Successor Shape

Contract `084` requires one end-to-end architecture that jointly owns:

- one continuous source/output timeline
- simultaneous material-dependent resolution or an equivalent unified
  representation
- transient classification, placement, phase treatment, and replica prevention
- tonal peak and dormant-state phase behavior
- shared linked-stereo decisions and preserved channel relationships
- exact boundaries, target length, bounded memory, and deterministic execution

Source studies of Rubber Band and Signalsmith remain useful for state ownership,
scheduling, guidance, and validation ideas. Signal implementation stays
clean-room and external engines remain comparators.

The rejected successor record is
`docs/architecture/offline-time-stretch-successor-brief.md`.
`EventSealedMultiresolutionPhaseField` failed pre-implementation structural
feasibility: its frozen 16-sample energy-rise tie rule places an isolated
impulse token `15` samples early while its gate requires the exact impulse
sample. Contract `084` Rule 7 closes this multiresolution phase-vocoder family.
Historical translation memos and the rejected brief are evidence only.

The non-phase-vocoder feasibility decision is
`docs/architecture/offline-time-stretch-non-phase-vocoder-feasibility.md`.
WSOLA cannot own arbitrary polyphony; the pinned direct-subband sinusoidal
specimen failed Signal's mono, long-form objective, linked-stereo, and exact
mechanics gates; deterministic sines/transients/noise lacks one complete
linked-channel recombination law; and reviewed neural synthesis does not meet
the target ratio, determinism, or first-party operating boundary. No successor
brief opens.

## Separate Creative Path

Intentional creative expansion is governed separately by Contract `085` and
`docs/architecture/offline-creative-time-stretch-study.md`. Its automatic
route and both overlaps are paused. Public `Dream` owns exact fixed `4x`,
`8x`, and `16x`. Public `Cyclic` owns exact `2x`, `4x`, and `8x` with one
`5..90 ms` cycle duration. Neither creative character replaces this renderer
or reopens Contract `084`. The attempted `LayeredCloud` owner closed without
promotion.

## Candidate Isolation

Successor work happens in a disposable branch or worktree. It does not add
modules, hidden review methods, report modes, or test scaffolding to `main`
before admission. One complete renderer runs the fixed structural gate, then
the synthetic gate, long-form mono pack, and independent linked-stereo review.

Failed candidate branches are removed. Their dominant failure is logged once.
Repeated narrow variants require architecture reassessment.

## Historical Record

The rejected structural hybrid, phase-gradient, H/R/P, frequency-adaptive,
direct multiscale, material-state, and stereo proof sequence is summarized in
roadmap `g10.029` and Contract `082`. The full pre-consolidation architecture
ledger remains in git history at `1d1b02f1`.

## Next Task

Retain this frozen baseline and keep its successor lane closed. `g10.031` also
retained the `2x..4x` creative overlap pause. `g10.032` later reopened Cyclic
research, admitted the accepted private renderer, and froze its public
fixed-ratio surface. Batch 32.28 admits the public wrapper. Execute `g10.032`
Batch 32.29 only. Keep both acoustic renderers, both overlaps, product routing,
and Contract `084` unchanged.
