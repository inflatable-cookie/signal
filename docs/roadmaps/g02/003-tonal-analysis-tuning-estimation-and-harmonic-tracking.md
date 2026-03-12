# Roadmap g02.003: Tonal Analysis, Tuning Estimation, and Harmonic Tracking

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.001
Vision tags: RES, DSP
Target envelope: deepen `signal-analysis-tonal` from whole-track key detection
into a more credible harmonic analysis surface with tuning and section-level
evidence.

## Problem

Signal can currently detect a global key from whole-track chroma, but that is
not enough for richer reuse:

1. tuning reference is caller-supplied rather than estimated,
2. no section-level key or harmonic change surface exists,
3. whole-track key confidence alone is too coarse for mixed or modulating
   material,
4. downstream tools would still need to improvise local harmonic evidence.

## Goals

- add tuning-estimation support to the tonal stack
- expose segment-level key and harmonic-change evidence
- keep the API confidence-led instead of label-led
- align tonal analysis to the shared substrate from `g02.001`

## Non-Goals

- full chord transcription or notation export
- melody extraction or source separation
- product-specific harmonic UX decisions

## Execution Plan

### 003.1 Tuning and chroma depth

- [x] add tuning-estimation support and explicit reference reporting
- [x] improve chroma and profile-scoring diagnostics where needed
- [x] freeze low/medium/high analysis tiers around the new surface

### 003.2 Segment-local tonal tracking

- [x] add section or windowed key-tracking outputs
- [x] expose harmonic-change or novelty markers where confidence supports them
- [x] make modulation and mixed-tonality ambiguity explicit in the API

### 003.3 Validation and evidence

- [x] add fixtures for stable-key, modulating, detuned, and weak-tonal-centre
      material
- [x] align docs and feature references to the deeper tonal surface
- [x] log closure evidence and remaining limits

## Acceptance Signals

1. `signal-analysis-tonal` can explain more than one global key label.
2. Tuning and local harmonic evidence are reusable without app-local helper
   code.
3. Weak or ambiguous tonality is surfaced explicitly rather than forced into a
   brittle best guess.

## Risks and Mitigations

- Risk: tonal scope balloons into full MIR transcription.
- Mitigation: stop at tuning, local key, and harmonic-change evidence.
- Risk: local key tracking becomes too noisy to trust.
- Mitigation: keep confidence and ambiguity thresholds explicit and fixture-led.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] fixtures for detuning, modulation, and low-confidence material
- [x] closeout notes comparing global and section-local outputs on the same
      inputs

## Current Evidence

The opening `g02.003` tranche moves `signal-analysis-tonal` past a bare
whole-track key label and into an explicit tuning/scoring surface:

- `signal-analysis-tonal` now exposes:
  - estimated or fixed tuning reference reporting
  - tuning confidence and runner-up tuning candidates
  - compact key-profile scoring diagnostics with best and runner-up candidates
- low/medium/high detector profiles now freeze explicit tuning-search
  defaults instead of leaving that behavior implicit
- fixture coverage now pins:
  - stable major/minor material under estimated tuning
  - detuned whole-track material with recovered reference near the source
  - fixed-reference reporting
  - non-native-rate stability under the frozen tonal substrate

The current local-tracking tranche now makes tonal motion visible across the
track instead of stopping at one global key estimate:

- `signal-analysis-tonal` now exposes:
  - section-local tonal segments with local key/confidence/scoring evidence
  - harmonic-change events between adjacent tonal segments
- the local surface preserves:
  - overlapping windowed key tracking on the same tuning-aware chroma path
  - explicit change timing, from/to key evidence, and chroma-distance support
- fixture coverage now also pins:
  - stable local key tracking on steady tonal material
  - a clear C-major to G-major modulation with explicit harmonic-change output

The closing ambiguity tranche makes weak or competing tonality explicit in the
same public surface instead of leaving callers to infer it from correlation
margins:

- `signal-analysis-tonal` now also exposes:
  - per-segment ambiguity summaries
  - track-level local ambiguity summaries for weak tonal centre, modulation,
    and mixed-tonality recurrence
- the local surface now preserves:
  - ambiguity timing and segment span context
  - explicit primary and alternate key evidence where the local state supports
    it
- fixture coverage now also pins:
  - weak-tonal-centre material without a credible stable local key
  - recurring `C -> G -> C` material as mixed tonality rather than one-way
    modulation

## Residual Scope

`g02.003` is complete for the current tonal target envelope.

Remaining deeper tonal work, if reopened later, should be treated as future
scope beyond this milestone:

- beat- or bar-aligned harmonic tracking instead of fixed local windows
- richer chord, function, or scale-degree interpretation
- corpus-backed calibration against more varied real-world modulation cases

## Next Task

Open `g02.004` with an explicit multichannel loudness aggregation contract and
broader true-peak/sample-rate behavior in `signal-analysis-loudness`, then pin
deterministic fallback behavior before deepening trace surfaces.
