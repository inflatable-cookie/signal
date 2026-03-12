# Roadmap g02.005: Transient, Timbral, and Descriptor Feature Packs

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.001
Vision tags: RES, DSP
Target envelope: deepen `signal-analysis-character` from a compact summary
surface into a stronger reusable descriptor stack for cataloging, search,
validation, and higher-level inference.

## Problem

Signal's character analyzer has a useful baseline, but it is still too thin to
serve as the shared descriptor substrate for later work:

1. many standard spectral descriptors are still absent,
2. transient and temporal-shape evidence remains coarse,
3. higher-level inference work would otherwise build on weak feature packs,
4. descriptor extraction is not yet organized as reusable bundles or tiers.

## Goals

- add richer timbral and spectral descriptor coverage
- deepen transient and temporal-shape extraction
- group descriptors into reusable feature packs instead of ad hoc fields
- keep outputs analyzable and confidence-backed for downstream reuse

## Non-Goals

- full MIR taxonomy coverage in one batch
- source separation or transcription
- product-specific tagging heuristics

## Execution Plan

### 005.1 Spectral descriptor depth

- [x] add practical spectral descriptors such as rolloff, flatness, contrast,
      and MFCC-adjacent surfaces where appropriate
- [x] keep descriptor definitions and reduction policy explicit
- [x] align extraction with the shared spectral substrate

### 005.2 Transient and temporal-shape depth

- [x] add stronger transient-marker or transient-density surfaces
- [x] capture useful attack/sustain/decay and dynamics-shape summaries where
      feasible
- [x] freeze a first descriptor-pack API instead of only one monolithic result

### 005.3 Validation and evidence

- [x] add deterministic fixture coverage for contrasting audio-character cases
- [x] document descriptor reductions and intended reuse boundaries
- [x] log closure evidence and remaining descriptor gaps

## Acceptance Signals

1. Signal exposes a materially richer descriptor stack than centroid/ZCR/RMS
   alone.
2. Descriptor extraction can feed both direct consumer use and later embedding
   work without local feature duplication.
3. Reduction policy is explicit enough to keep outputs comparable across runs.

## Risks and Mitigations

- Risk: descriptor count grows without a coherent API.
- Mitigation: group outputs into named packs and tiers rather than appending
  endless flat fields.
- Risk: descriptors become expensive without clear reuse value.
- Mitigation: require at least one concrete downstream reuse story per added
  descriptor family.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] fixtures for contrasting transient, noisy, tonal, and sustained material
- [x] explicit descriptor-pack examples recorded at closeout

## Next Task

Open `g02.006` by defining the first embedding and semantic-inference baseline
on top of the shared descriptor packs rather than app-local features.
