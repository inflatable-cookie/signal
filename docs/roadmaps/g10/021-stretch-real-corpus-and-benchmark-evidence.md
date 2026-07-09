# 021 - Stretch Real Corpus And Benchmark Evidence

Status: complete
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
- [x] add a runner that produces deterministic comparison reports for draft,
  OfflineHighQuality, and optional external benchmark output
- [x] add Rubber Band CLI output as a behavioral benchmark option only; do not
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

- [x] optional Rubber Band CLI render comparison path with clean-room source
  boundary documented

## Acceptance Criteria

- [x] synthetic reports remain available for fast local tests
- [x] real-corpus report output can be saved as an artifact and compared across
  runs
- [x] benchmark comparison is optional and clean-room
- [x] no product-facing gate depends on unaudited external source code

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
- 2026-07-07: added optional `--external-benchmark-render` support for
  operator-supplied rendered WAV outputs. Signal records external tool identity,
  rendered metadata, timing drift when the case maps to Signal-generated source,
  and the rendered-output-only clean-room boundary. No Rubber Band source,
  library, or dependency was added.
- 2026-07-07: added `fma-stretch-corpus-select` for local-only FMA large-bundle
  listening candidates. It reads FMA metadata with license/provenance fields,
  verifies local MP3 paths, classifies candidates into stretch corpus families,
  and writes a generated manifest under `target/` without copying or committing
  audio.
- 2026-07-07: connected local listening source manifests to
  `stretch-corpus-report`. FMA selection can now emit TSV, and the report runner
  verifies local paths, records source/provenance rows, reduces missing required
  source counts, and creates listening-note slots against actual local files
  without decoding, copying, or committing licensed audio.
- 2026-07-07: added an FMA review-seed TSV output for no-listening runs. The
  seed keeps fixed per-family coverage and avoids repeated artists where
  possible. It is explicitly coverage evidence, not a subjective quality
  judgment.
- 2026-07-07: ran the local report with the FMA review-seed TSV:
  `operator_listening_sources=10`, `missing_assets=0`, no external benchmark
  comparisons. This proves local real-source coverage wiring, not listened
  quality.

## Next Task

Replace the review seed with listened curation and/or add optional external
rendered-output comparisons before using real-source evidence for promotion.
