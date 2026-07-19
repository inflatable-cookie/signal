# Offline Time-Stretch Synthesis

Status: active baseline; successor architecture frozen
Owner: dsp
Updated: 2026-07-19
Contract refs: `046`, `084`; historical evidence `082`
Roadmap ref: `g10.030`

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

The complete successor is frozen in
`docs/architecture/offline-time-stretch-successor-brief.md` as
`SourceAnchoredMultiresolutionPhaseField`. It uses three simultaneous,
non-overlapping frequency-owned STFT scales on one absolute map, one-shot
attack reassignment, coherent tracked tonal phase, dormant/reactivation state,
and native-channel linked synthesis. That brief is the implementation
authority; historical translation memos are evidence only.

## Candidate Isolation

Successor work happens in a disposable branch or worktree. It does not add
modules, hidden review methods, report modes, or test scaffolding to `main`
before admission. One complete renderer runs the fixed structural gate, then
the long-form mono pack, then independent linked-stereo review.

Failed candidate branches are removed. Their dominant failure is logged once.
Repeated narrow variants require architecture reassessment.

## Historical Record

The rejected structural hybrid, phase-gradient, H/R/P, frequency-adaptive,
direct multiscale, material-state, and stereo proof sequence is summarized in
roadmap `g10.029` and Contract `082`. The full pre-consolidation architecture
ledger remains in git history at `1d1b02f1`.

## Next Task

Run `g10.030` Batch 30.3 in one disposable branch or worktree. Implement the
frozen successor without adding candidate surfaces to `main`.
