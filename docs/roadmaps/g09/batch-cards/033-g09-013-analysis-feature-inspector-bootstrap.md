# 033 - g09.013 Analysis Feature Inspector Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/013-dsp-graph-analysis-interactive-demo-suite-and-audit-closeout-proof.md`
Governing contracts: `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`, `docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md`, `docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`

## Objective

Bootstrap one repo-owned `signal.demo.analysis.feature-inspector` surface that
wraps the existing analysis examples and frozen proof posture into a single
operator-facing demo family without inventing plugin browsing, asset-library UX,
or a new downstream workflow shell.

## Scope

- add one shared analysis demo manifest, launch task, scenario notes, and
  machine-readable receipt under `demos/`
- reuse the existing rhythm, tonal, and loudness example binaries as the
  starting live probes
- add the minimum bounded shared analysis entry point needed to expose
  character and semantic posture inside the same demo family
- keep the demo explicitly offline and synthetic-input oriented
- update coverage and strict-lane currentness surfaces to mark the analysis
  crates live only if the demo really launches

## Out Of Scope

- plugin capability browsing
- asset scan-root or library browsing workflows
- new host, runtime, or plugin demo shells
- widening semantic or rhythm algorithms beyond the already-closed contracts
- replacing existing acceptance or corpus automation

## Acceptance Criteria

- `signal.demo.analysis.feature-inspector` exists as a repo-owned live demo
  surface
- the demo exposes bounded operator posture for:
  - rhythm
  - tonal
  - loudness
  - character
  - semantic analysis
- the demo receipt records covered crates, covered scenarios, validation
  commands, and explicit exclusions
- `demos/coverage-matrix.md` and `demos/coverage-matrix.json` mark the covered
  analysis crates live only after the demo works

## Validation

- `effigy demo:analysis-feature-inspector`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- demo manifest under `demos/manifests/`
- launch wrapper under `demos/scripts/`
- operator scenario under `demos/scenarios/`
- generated receipt under `demos/receipts/`
- batch log with validation actually run

## Stop Conditions

- the work starts depending on demo-owned asset browsing or scan-root planning
- the missing character or semantic posture cannot be exposed through one
  bounded shared analysis entry point
- the batch starts wanting a new product shell instead of a proof-oriented demo

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the
remaining `g09.013` work is now tightly batch-cardable as audit closeout proof
or should stay in planning pause until that seam is clearer.
