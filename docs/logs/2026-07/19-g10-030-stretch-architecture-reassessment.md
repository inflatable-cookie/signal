# g10.030 Stretch Architecture Reassessment

Date: 2026-07-19
Status: complete
Roadmap: `g10.030` Batch 30.4

## Change

Replaced the rejected centered-detector successor with one complete
`EventSealedMultiresolutionPhaseField` brief. No DSP, fixture, report mode, or
candidate API changed on `main`.

The replacement resolves the Batch 30.3 cause as one connected schedule:

- adjacent detector blocks search the full future block that produced novelty
- fixed lookahead finalizes exact source event samples before synthesis
- every event becomes an analysis centre on the absolute source/output map
- deterministic source/output seals make every non-anchor window zero at the
  source event and mapped output sample
- the anchor alone owns the source attack; short-scale attack reassignment and
  linked phase reset occur there once

The tonal, simultaneous-resolution, linked-stereo, boundary, exact-length,
fixed-memory, deterministic, gate, rejection, cleanup, and admission rules are
frozen in the same brief. There is no detector-radius, threshold, reset-tick,
selector, or row-specific repair lane.

## Decision

Batch 30.5 may implement exactly one disposable candidate. Structural
admission now includes exact impulse tokens and one non-zero analysis-window
owner per event sample. A second event-placement or replica failure closes this
multiresolution phase-vocoder family under Contract `084` Rule 7.

## Boundary

- started clean on `main` at `a224ea8c`
- documentation only
- production OfflineHighQuality and retained harness output unchanged
- RealtimePreview, render-plane integration, Loophole, and Chorus untouched
- no implementation begins in Batch 30.4

## Next Task

Create one disposable Batch 30.5 worktree from this commit. Implement the
frozen brief exactly and stop after structural and synthetic admission decide
whether long-form listening audio may be generated.
