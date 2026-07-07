# 021 - Stretch Real Corpus And Benchmark Evidence

Status: ready
Owner: dsp
Created: 2026-07-07
Depends on: g10.015, g10.020
Vision tags: `DSP`, `STRETCH`, `EVIDENCE`

## Problem

OfflineHighQuality currently promotes from repository-local synthetic
comparison evidence. That is useful for fast regression checks, but it is not
enough to claim Rubber Band-class behavior. The next stretch work needs real
material, repeatable reports, optional external benchmark output, and listening
notes before more product-facing quality claims land.

## Goals

- [x] define the first checked-in corpus manifest shape for drums/percussion,
  bass, vocals, pads/sustains, full mixes, tempo ramps, loop seams, and extreme
  ratios without committing licensed source audio
- [ ] add a runner that produces deterministic comparison reports for draft,
  OfflineHighQuality, and optional external benchmark output
- [ ] add Rubber Band CLI output as a behavioral benchmark option only; do not
  add source or library dependency
- [x] record listening-note slots next to objective metrics so operator review
  can capture artifacts the metrics miss
- [x] make the report name, engine version, corpus id, ratio/pitch curves, and
  projection epoch visible in the output

## Execution Plan

### Batch 21.1 - Corpus Manifest

- [x] manifest schema, source-audio policy, corpus family coverage, and
  missing-asset behavior

### Batch 21.2 - Report Runner

- [x] deterministic report command for draft and OfflineHighQuality outputs
- [x] saved report artifact with objective metrics and listening-note slots

### Batch 21.3 - External Benchmark Option

- [ ] optional Rubber Band CLI render comparison path with clean-room source
  boundary documented

## Acceptance Criteria

- [x] synthetic reports remain available for fast local tests
- [x] real-corpus report output can be saved as an artifact and compared across
  runs
- [ ] benchmark comparison is optional and clean-room
- [ ] no product-facing gate depends on unaudited external source code

## Validation

- `cargo test -p signal-dsp-stretch`
- focused report-runner command once the runner exists
- `effigy qa:docs` when report docs change

## Progress

- 2026-07-07: opened as active g10 stretch evidence work. Time-stretch is not
  a deferred backlog item.
- 2026-07-07: implemented `STRETCH_CORPUS_MANIFEST` with source policy,
  missing-asset behavior, fixture path rules, and checked-in manifest docs under
  `fixtures/stretch-corpus/`. Licensed listening audio remains local-only.
- 2026-07-07: implemented `stretch-corpus-report`, deterministic report
  formatting, missing licensed-asset rows, objective synthetic comparison rows,
  ratio/pitch curve fields, projection epoch output, and listening-note slots.

## Next Task

Implement Batch 21.3: optional external benchmark render comparison path with a
documented clean-room source boundary.
