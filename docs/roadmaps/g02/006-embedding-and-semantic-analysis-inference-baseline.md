# Roadmap g02.006: Embedding and Semantic-Analysis Inference Baseline

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.001, g02.005
Vision tags: RES, DSP
Target envelope: open the first reusable Signal-owned embedding and semantic
analysis path without dragging model training or app-specific classification
policy into every consumer.

## Problem

Signal research already points toward an embedding-oriented analysis surface,
but no reusable implementation boundary exists yet:

1. semantic or catalog inference would otherwise land in app-local repos,
2. no stable `signal-analysis-embed` crate or contract exists,
3. descriptor packs from lower-level analyzers have no shared inference
   consumer,
4. later ML work would lack a bounded CPU/runtime contract.

## Goals

- define the first `signal-analysis-embed` crate boundary
- add one practical embedding or semantic-inference path
- keep model/runtime constraints explicit and host-neutral
- build on descriptor packs rather than bypassing them

## Non-Goals

- training infrastructure or dataset ownership
- final product taxonomy or recommendation policy
- mandatory GPU or remote inference

## Execution Plan

### 006.1 Contract and crate boundary

- [x] author the first embedding/inference crate and public result contracts
- [x] define model loading, versioning, and fallback expectations
- [x] keep resource and determinism constraints visible

### 006.2 First inference path

- [x] implement one practical embedding or semantic-analysis path
- [x] validate descriptor-pack integration rather than hidden local features
- [x] expose confidence or distance diagnostics where meaningful

### 006.3 Validation and evidence

- [x] add fixture-backed smoke coverage for inference outputs and failure modes
- [x] document model assumptions and portability limits
- [x] log closure evidence and remaining inference gaps

## Acceptance Signals

1. Signal owns a reusable semantic-analysis boundary instead of deferring it to
   product repos.
2. Inference inputs and outputs are explicit enough that consumers can trust
   the contract even if models change later.
3. Resource constraints and failure modes are visible rather than implicit.

## Risks and Mitigations

- Risk: ML scope overwhelms the rest of the Signal roadmap.
- Mitigation: freeze one bounded inference slice and defer training/product
  policy explicitly.
- Risk: inference bypasses lower-level analyzers and duplicates feature work.
- Mitigation: require integration with shared descriptor packs wherever
  feasible.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] fixture-backed inference smoke tests
- [x] closeout notes on model/runtime portability assumptions

## Next Task

Open `g02.007` by defining the first shared analysis corpus layout and harness
entry points for regression-sensitive analyzer families.
