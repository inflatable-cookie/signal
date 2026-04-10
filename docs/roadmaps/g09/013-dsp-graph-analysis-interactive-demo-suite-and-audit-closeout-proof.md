# 013 - DSP, Graph, Analysis Interactive Demo Suite And Audit Closeout Proof

Status: active
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

#### Batch 13.1 Tranche 1 Outcome

- completed the graph-routing slice with the live
  `signal.demo.graph.execution-inspector` surface
- wrapped the existing multichannel, sidechain, multi-bus, and spatial
  descriptor plus acceptance family into one repo-owned demo manifest, script,
  scenario file, receipt, and Effigy task
- repaired stale focused acceptance wiring in the graph-routing proof family so
  the existing boundaries run honestly through the shared demo wrapper
- left the next tranche unpromoted because DSP processing-lab versus analysis
  feature-inspector still needs fresh planning judgment

#### Batch 13.1 Tranche 2 Ready Posture

- the next honest seam is the bounded DSP processing-lab bootstrap
- it will wrap the existing stretch, marker-analysis, and
  transform-artifact boundary family into one repo-owned DSP demo surface
- analysis feature-inspector remains deferred until its multi-crate operator
  posture is planned more explicitly

#### Batch 13.1 Tranche 2 Outcome

- completed the DSP slice with the live `signal.demo.dsp.processing-lab`
  surface
- wrapped the existing stretch, marker-analysis, and transform-artifact
  descriptor plus acceptance family into one repo-owned demo manifest, script,
  scenario file, receipt, and Effigy task
- repaired stale focused acceptance wiring in the DSP proof family so the
  existing boundaries run honestly through the shared demo wrapper
- left the next tranche unpromoted because analysis feature-inspector still
  needs a clearer single-surface operator posture before another honest ready
  card is claimed

#### Batch 13.2 Tranche 1 Ready Posture

- the next honest seam is the bounded analysis feature-inspector bootstrap
- it will wrap the existing rhythm, tonal, and loudness example binaries into
  one repo-owned analysis demo family
- it will add the minimum shared analysis entry point needed to expose
  character and semantic posture inside that same bounded demo family
- it must stay offline and synthetic-input oriented rather than widening into
  plugin browsing or asset-library workflow design

#### Batch 13.2 Tranche 1 Outcome

- completed the analysis slice with the live
  `signal.demo.analysis.feature-inspector` surface
- wrapped the existing rhythm, tonal, and loudness example binaries plus one
  new shared character-and-semantic inspector example into one repo-owned demo
  manifest, script, scenario file, receipt, and Effigy task
- promoted the shared analysis crates to live demo coverage in the workspace
  coverage matrix because the manifest and receipt now exist
- left the next tranche unpromoted because the remaining `g09.013` audit
  closeout proof still wants fresh planning judgment rather than another honest
  auto-ready card

#### Batch 13.3 Tranche 1 Ready Posture

- the next honest seam is the bounded audit closeout proof bundle
- it will compile the final live demo coverage, explicit deferred demo scope,
  and the `g09` handoff posture without inventing another demo binary
- it must stay additive over the existing manifests, receipts, Effigy tasks,
  and coverage matrix instead of reopening implementation work

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

Continue the active strict `g09` lane from
`docs/specs/batch-cards/034-g09-013-audit-closeout-proof-bundle.md`.
