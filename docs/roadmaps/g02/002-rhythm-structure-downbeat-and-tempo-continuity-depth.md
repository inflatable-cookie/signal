# Roadmap g02.002: Rhythm Structure, Downbeat, and Tempo-Continuity Depth

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.001
Vision tags: RES, DSP
Target envelope: deepen `signal-analysis-rhythm` from beat/tempo detection into
musically useful rhythm-structure analysis with explicit ambiguity and tempo
continuity evidence.

## Problem

Signal's rhythm analyzer already produces more than a toy BPM estimate, but it
still stops short of the structure-level outputs that downstream tools need:

1. downbeat and meter confidence are still shallow,
2. tempo continuity and segment-level change evidence need a stronger public
   surface,
3. streaming and offline rhythm outputs are not yet aligned on one substrate,
4. later beat-grid or timeline consumers would still need app-local repair
   heuristics.

## Goals

- add stronger meter/downbeat inference and ambiguity reporting
- surface tempo-segment and continuity summaries explicitly
- align rhythm analysis to the shared substrate from `g02.001`
- keep confidence and fallback reasoning first-class

## Non-Goals

- final product-specific beat-grid editing semantics
- genre-specific heuristics baked into the core API
- end-to-end DAW transport ownership in this batch

## Execution Plan

### 002.1 Meter and downbeat depth

- [x] deepen public meter/downbeat outputs and diagnostics
- [x] preserve ambiguity surfaces for half/double-time and competing meter
      cases
- [x] add fixtures that explicitly stress weak-accent and syncopated material

### 002.2 Tempo continuity

- [x] expose segment-local tempo and continuity summaries as stable public types
- [x] distinguish stable tempo, localized damage, and mid-track drift cases
- [x] keep recommendation and trust outputs explicit rather than implicit

### 002.3 Shared-substrate alignment and evidence

- [x] adopt the shared resampling/spectral substrate where it improves
      consistency
- [x] add offline and bounded-streaming validation where practical
- [x] log the closure evidence and remaining rhythm limits

## Acceptance Signals

1. `signal-analysis-rhythm` can report useful downbeat/meter and tempo
   continuity evidence instead of only a top-line BPM.
2. Ambiguity is visible in the API rather than hidden in internal rescoring.
3. Rhythm analysis can serve both rapid scan and detailed analysis use cases
   without duplicating substrate logic.

## Risks and Mitigations

- Risk: rhythm APIs become overfit to one product's grid UX.
- Mitigation: keep outputs descriptive and diagnostic, not editor-specific.
- Risk: every difficult tempo case expands scope indefinitely.
- Mitigation: freeze one practical structure-analysis slice and defer exotic
  meter work explicitly.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] fixtures covering straight, swung, ambiguous, and variable-tempo cases
- [ ] validation commands and example outputs captured at closeout

## Current Evidence

The opening `g02.002` tranche established the first compact structure-level
surface on top of the existing rhythm inference:

- `signal-analysis-rhythm` now exposes a `RhythmStructureSummary` derived from
  the inferred meter state rather than forcing timeline consumers to rebuild
  bar spans from raw downbeat arrays and recovery metadata
- the summary surfaces:
  - bar spans
  - whole-track vs recovery-window vs extrapolated support
  - compact continuity state for bar-length and downbeat-phase handling
- fixture coverage now pins both:
  - stable whole-track bar structure
  - recovery-backed segment structure

The current tranche extends that surface so difficult meter cases are visible
through one explicit assessment contract instead of remaining hidden in
internal rescoring:

- `signal-analysis-rhythm` now exposes:
  - `RhythmStructureAmbiguitySummary`
  - `RhythmStructureFallbackSummary`
  - `RhythmStructureAssessment`
- the assessment preserves:
  - primary and runner-up structure candidates
  - explicit ambiguity kind for weak accent, competing meter, downbeat-phase
    competition, and recovery-window fallback
  - fallback guidance derived from the current meter-state lifecycle
- fixture coverage now also pins:
  - weak-accent material that still yields a usable structure
  - competing-meter material with visible runner-up contrast
  - pickup-extension/downbeat-phase ambiguity
  - meterless recovery-window fallback behavior

The current tempo tranche now makes the existing tempo diagnostics and
continuity policy consumable through one compact summary rather than requiring
callers to interpret multiple analyzer-internal structures:

- `signal-analysis-rhythm` now exposes:
  - `TempoSegmentSummary`
  - `TempoContinuitySummary`
  - `TempoStructureSummary`
  - `BeatAnalysisResult::tempo_structure_summary()`
- the summary preserves:
  - coarse whole-track, edge-trimmed-stable, and stable-core tempo regions
  - current trust/recommendation plus selected tempo and fallback state
  - continuity action, severity, arc, trigger, and beat-based fallback timing
- coverage now pins:
  - whole-track stable click-track locking
  - localized edge-damage structure with separate stable regions
  - real core-stable monitoring behavior
  - mid-track unstable clear behavior

The closing tranche adds the bounded-window evidence that was still missing:

- bounded trailing-window validation now pins that:
  - stable structure and tempo summaries remain actionable across practical
    bounded windows
  - weak-accent material still surfaces explicit ambiguity and usable tempo
    decisions under the same bounded-window constraint
- this gives `g02.002` both:
  - full offline fixture coverage
  - practical bounded-stream validation without inventing a new runtime-facing
    incremental API inside the milestone

## Residual Scope

None inside `g02.002`. The milestone now has the intended public structure,
ambiguity, and tempo continuity surfaces plus bounded-window validation.

## Next Task

Open `g02.003` with a tuning-estimation and chroma-depth batch in
`signal-analysis-tonal`, then use that surface to start local harmonic
tracking without reopening `g02.002`.

## Completion

`g02.002` is complete.
