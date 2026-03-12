# Roadmap g02.007: Analysis Corpus, Benchmarking, and Acceptance Harnesses

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.002, g02.003, g02.004, g02.005, g02.006
Vision tags: RES, DSP, RT
Target envelope: make Signal's deeper analysis stack defensible by adding one
shared benchmarking and acceptance surface instead of relying on scattered
examples and ad hoc fixture checks.

## Problem

Once the deeper analysis crates land, Signal needs stronger quality protection:

1. algorithm changes will be hard to compare without shared corpus baselines,
2. examples alone are too weak to protect regression-sensitive metrics,
3. downstream consumers need evidence on accuracy, stability, and performance,
4. the next generation should not start without a stronger validation spine.

## Goals

- create one shared analysis corpus and benchmark posture
- add repeatable acceptance harnesses for core analyzer families
- record accuracy, confidence, and performance evidence in one place
- keep the harness usable by a dedicated Signal thread without app-local glue

## Non-Goals

- final public benchmark marketing claims
- exhaustive MIR research benchmarking
- replacing unit tests with corpus harnesses

## Execution Plan

### 007.1 Corpus and harness shape

- [x] define the first shared corpus layout and fixture taxonomy
- [x] add reusable harness entry points for analyzer comparison and regression
      checks
- [x] keep licensing and artifact-size constraints explicit

### 007.2 Metrics and thresholds

- [x] freeze practical accuracy/stability/performance metrics per analyzer
      family
- [x] add threshold or drift-reporting policy without overfitting to one corpus
- [x] surface confidence and ambiguity calibration where meaningful

### 007.3 Evidence and generation closeout

- [x] log benchmark and acceptance evidence for the active analyzer families
- [x] record deferred gaps and backlog candidates for the next generation
- [x] close `g02` only once the acceptance spine is credible

## Acceptance Signals

1. Signal can compare analyzer revisions against a shared baseline corpus.
2. Performance and quality evidence are recorded in a repeatable way instead of
   scattered through examples.
3. The next generation can start from protected algorithm outputs rather than
   loosely described behavior.

## Risks and Mitigations

- Risk: benchmark work becomes a giant dataset project.
- Mitigation: start with a small but representative corpus and expand only when
  a concrete regression class appears.
- Risk: thresholds become fake precision.
- Mitigation: report drift clearly and use bounded practical thresholds.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] corpus layout and harness commands recorded in closeout evidence
- [x] explicit residual-risk note for analyzer families not yet benchmarked

## Next Task

`g02.007` is complete. Open a new generation only when Signal needs a new
sequenced continuation queue beyond the now-closed `g02` analysis spine.
