# Rubber Band Behavioural Forensics

Status: superseded by `004-source-studied-stretch-architecture.md`
Memo: `g10.029` algorithm-class reset
Owner: dsp
Last updated: 2026-07-12
Related roadmap: `g10.029`

## 1) Problem

Signal's successor research repeatedly froze individual mechanisms while
forbidding local timing redistribution, transient phase reset, joint detector
and synthesis tuning, and simultaneous multi-resolution processing. Those
constraints excluded documented Rubber Band-class behaviour before Signal
tested a comparable complete system.

The resulting rejections prove that the constrained candidates failed. They do
not prove that Signal-native parity is unavailable.

## 2) External Evidence

Rubber Band's public R2 notes describe four interacting mechanisms:

- one block phase-vocoder path
- phase resets on percussive transients
- adaptive stretch between reset points
- vertical phase lamination

Its integration notes state that the requested ratio is a long-term average;
the effective local ratio varies around detected features. Offline mode studies
the complete source before synthesis. The public API exposes the calculated
output increments, phase-reset curve, and exact-time points.

R3 differs internally from R2. Its standard-window mode uses a full
multi-resolution processing scheme. Short-window mode disables that scheme and
uses one resolution, trading quality for speed and lower delay. The public API
also states that R3 improves complex mixes, vocals, soft onsets, smooth pitch
changes, and bass-heavy material—the same broad regions where Signal's current
candidate retains grain, softness, and occasional transient spikes.

## 3) Root-Cause Hypotheses

1. Signal's exact global source-to-output map suppresses the event-local timing
   freedom needed to preserve attacks while meeting duration globally.
2. Identity phase locking without coordinated transient phase treatment moves
   or smears attacks when resolution and hops change. The oracle candidate's
   `1.5x` impulse error is direct evidence.
3. Selecting one window per frame discards cross-resolution information. A
   simultaneous representation may preserve long tonal structure and short
   attack structure without time-domain branch switching.
4. Detector-only gates are not predictive enough. Detection, time allocation,
   phase treatment, and synthesis must be evaluated as one causal system.
5. Strict per-metric non-regression rejected candidates before bounded tuning
   against the actual listening defects. Objective measures remain safety and
   attribution evidence, not independent proof of musical quality.

## 4) Recommendation

Do not start another Signal synthesis candidate yet. First turn Rubber Band R2
and R3 into behavioural specimens through generated controls and existing
licensed listening sources.

Measure:

- global duration and event-local output increments
- impulse, dense-event, boundary-event, and soft-onset placement
- pre/post-event time compensation
- transient crest, replicas, and phase-coherence signatures
- steady-tone, two-tone, chirp, bass, vocal, and complex-mix texture
- R2 transient/detector/phase option deltas
- R3 standard multi-resolution versus R3 short single-resolution deltas
- linked versus independent channel behaviour

Use the smallest probe set that distinguishes the hypotheses. Do not infer R3
internals from one waveform. Require repeatable signatures across ratios and
control families.

## 5) Boundary

The first pass uses public documentation, public API introspection, and
black-box rendered outputs. Do not copy Rubber Band source or implementation
expressions into Signal. Any direct GPL source study is a separate
operator/legal decision and must remain isolated from implementation until its
reuse boundary is explicit.

That operator decision was made on 2026-07-13. Memo 004 replaces source
exclusion with a pinned-revision, provenance-controlled architecture study.
GPL expression and unexplained constants remain outside Signal.

## 6) Promotion

Promoted into:

- `docs/architecture/offline-time-stretch-synthesis.md`
- contract `082`, Rule 29
- roadmap `g10.029`, Batches 29.6BD through 29.6BG

## Sources

- [Rubber Band technical notes](https://breakfastquay.com/rubberband/technical.html)
- [Rubber Band integration notes](https://breakfastquay.com/rubberband/integration.html)
- [Rubber Band stretcher API](https://breakfastquay.com/rubberband/code-doc/classRubberBand_1_1RubberBandStretcher.html)

## Next Task

Prove the simultaneous `512/2048/8192` union frame and exact identity dual
before study, schedule, phase modification, or tuning.
