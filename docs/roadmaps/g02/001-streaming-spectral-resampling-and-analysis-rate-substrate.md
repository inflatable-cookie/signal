# Roadmap g02.001: Streaming Spectral, Resampling, and Analysis-Rate Substrate

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g01.005, g01.007
Vision tags: RES, RT, DSP
Target envelope: establish the shared spectral and sample-rate substrate that
all deeper Signal analyzers can reuse without open-coding their own mono
conversion, truncation, framing, or rate-conversion logic.

## Problem

Signal now has useful offline analyzers, but they still lean on whole-buffer
offline assumptions:

1. no shared streaming/incremental STFT path exists,
2. analysis crates do not share a reusable analysis-rate conversion boundary,
3. mono reduction and windowing choices are analyzer-local,
4. later rhythm, tonal, loudness, and descriptor work would otherwise deepen
   on top of duplicated framing and resampling logic.

## Goals

- add one reusable analysis-rate/resampling substrate for offline and future
  streaming analyzers
- extend `signal-dsp-spectral` with incremental spectral framing rather than
  only whole-buffer transforms
- define shared channel-reduction and analysis-window policy where needed
- keep the substrate host-independent and reusable outside Loophole runtime

## Non-Goals

- solve product-level analysis job orchestration
- introduce file I/O or asset-catalog ownership into Signal
- broaden into ML inference in this batch

## Execution Plan

### 001.1 Shared analysis-rate conversion

- [x] introduce the first shared resampling/analysis-rate crate or module
- [x] freeze reusable APIs for offline block conversion and bounded streaming
      rate adaptation
- [x] document supported quality tiers and determinism constraints

### 001.2 Incremental spectral surface

- [x] add streaming or chunked STFT framing to `signal-dsp-spectral`
- [x] expose reusable frame, hop, and overlap semantics for downstream
      analyzers
- [x] keep phase, window, and channel-policy choices explicit and testable

### 001.3 Contract and validation pass

- [x] align `signal-analysis` traits with the new substrate where needed
- [x] add targeted fixture coverage for resampling and incremental spectral
      equivalence against offline surfaces
- [x] log the closure evidence and residual limits

## Acceptance Signals

1. Rhythm, tonal, loudness, and descriptor analyzers can all consume one shared
   analysis-rate and spectral substrate instead of carrying local copies.
2. Signal exposes a credible streaming-ready spectral API without forcing host
   or runtime policy into DSP crates.
3. Offline and incremental spectral outputs are close enough to support shared
   validation fixtures and future streaming analyzers.

## Risks and Mitigations

- Risk: substrate work becomes a vague utilities bucket.
- Mitigation: only promote APIs already needed by at least two analyzer
  families.
- Risk: resampling policy leaks host/runtime assumptions into reusable crates.
- Mitigation: keep the boundary purely signal-domain and fixture-validated.

## Evidence Requirements

- [x] meaningful logs under `docs/logs/YYYY-MM/`
- [x] fixture-backed equivalence checks for offline vs incremental spectral
      outputs
- [x] validation commands actually run and recorded at closeout

## Current Evidence

The current batch established the first reusable substrate without closing the
entire milestone:

- `signal-dsp-resample` now owns deterministic offline and chunked mono
  resampling with explicit nearest and linear quality modes
- `signal-analysis` now exposes shared analysis input preparation for mono
  reduction, center trimming, and optional target-rate conversion
- `signal-dsp-spectral` now exposes `StreamingStft`, and the existing offline
  STFT path is validated against the same chunked framing semantics
- rhythm, tonal, and character analyzers now consume the shared preparation
  path instead of carrying their own mono/truncation glue
- loudness now consumes the same shared preparation boundary and freezes a
  48 kHz analysis-rate contract instead of silently degrading on non-48k input

## Closure Notes

`g02.001` is complete.

The current Signal analyzer surface now shares one reusable preparation and
rate-conversion boundary:

- rhythm, tonal, character, and loudness analyzers all consume
  `signal-analysis` input preparation instead of carrying local mono/truncate
  glue
- loudness, rhythm, tonal, and character profiles now freeze explicit analysis
  sample-rate defaults instead of inheriting the source rate implicitly
- incremental and offline STFT paths share validated framing semantics
- deterministic offline and chunked resampling are established as reusable DSP
  substrate, not analyzer-local helpers

Residual future work now belongs to downstream milestones rather than this
substrate milestone:

- rhythm/deeper tempo structure belongs in `g02.002`
- tonal/tuning/harmonic tracking belongs in `g02.003`
- any future substrate expansion should only reopen when a later milestone
  proves a new shared requirement

## Next Task

Open `g02.002` and deepen rhythm structure on top of the frozen shared
analysis-rate and spectral substrate instead of revisiting analyzer input
plumbing.
