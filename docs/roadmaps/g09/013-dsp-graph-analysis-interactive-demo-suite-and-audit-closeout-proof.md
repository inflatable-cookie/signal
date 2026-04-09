# 013 - DSP, Graph, Analysis Interactive Demo Suite And Audit Closeout Proof

Status: draft
Owner: core-product
Created: 2026-04-08
Depends on: g09.011
Vision tags: `DEMOS`, `DSP`, `ANALYSIS`, `CLOSEOUT`
Contract refs: `076`, `077`, `078`, `079`

## Problem

Even after the execution-facing demos land, Signal still needs an explicit proof
surface for the DSP, graph, and analysis crates that are otherwise only visible
through tests and benchmarks.

## Goals

- [ ] deliver interactive DSP, graph, and analysis demos that cover the active
      crate set
- [ ] use those demos to close the main audit-remediation proof gap
- [ ] finish `g09` with an explicit crate-coverage and deferred-scope record

## Non-Goals

- [ ] no attempt to build educational product experiences
- [ ] no replacement of corpus, benchmark, or acceptance automation

## Execution Plan

### Batch 13.1 - DSP And Graph Demo Paths

- [ ] add resampling, automation/control, and graph-routing scenarios that show
      input, output, and quality or topology receipts live
- [ ] cover multichannel, sidechain, bus, and spatial graph meaning where the
      crate claims support
- [ ] export manifests and sample outputs for each scenario

### Batch 13.2 - Analysis Demo Paths

- [ ] add rhythm, tonal, loudness, character, and semantic-analysis scenarios
- [ ] expose both normal and degraded-path analysis posture where the crates now
      support typed degraded outcomes
- [ ] link demos to corpus or benchmark evidence where the crates claim
      calibrated or fidelity-sensitive behavior

### Batch 13.3 - Audit Closeout Proof

- [ ] compile a crate-coverage matrix across all demo binaries and scenarios
- [ ] record unresolved deferred scope after `g09`
- [ ] define the final remediation proof bundle and the next generation handoff

## Acceptance Criteria

- [ ] DSP, graph, and analysis crates have repo-owned interactive proof paths
- [ ] demo coverage and deferred scope are explicit at generation closeout
- [ ] `g09` can close with evidence about what Signal really does today

## Risks And Mitigations

- Risk: demos hide the same limitations the audit was meant to expose.
- Mitigation: require manifests to record exclusions and degraded behavior
  explicitly.

- Risk: closeout becomes prose-only again.
- Mitigation: require coverage matrices, demo receipts, and focused validation
  commands as part of the proof bundle.

## Evidence Requirements

- [ ] log each DSP/graph/analysis demo tranche
- [ ] run the domain demo launch tasks and record manifest output
- [ ] run `effigy health`
- [ ] run `effigy qa:docs`
- [ ] capture the final `g09` deferred-scope record

## Next Task

After the preceding milestones are materially complete, close `g09` with an
explicit remediation verdict and open the next generation only for the still-
deferred hard tail.
