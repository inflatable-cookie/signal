# 2026-03-11 22:03:33 GMT - g02.007 corpus layout and harness opening tranche

Opened `g02.007` by defining the first shared analysis corpus layout and the
initial reusable acceptance/regression harness surface for Signal analyzers.

This tranche matters because the analysis stack now has one repo-owned place to
record fixture taxonomy, corpus constraints, and baseline-versus-candidate
checks instead of scattering that protection across crate-local tests and
examples.

Implemented changes:

- added shared corpus metadata and harness contracts in
  `crates/signal-analysis/src/lib.rs`, including:
  - `AnalysisCorpusFamily`
  - `AnalysisCorpusCaseMetadata`
  - `AnalysisCorpusCase`
  - `AnalysisMetricValue`
  - `AcceptanceThreshold`
  - `RegressionDriftLimit`
  - `AcceptanceHarnessReport`
  - `RegressionHarnessReport`
  - `run_audio_acceptance_harness()`
  - `compare_audio_analyzers()`
- added focused harness coverage in `signal-analysis` tests for:
  - passing acceptance thresholds
  - failing acceptance thresholds
  - baseline-versus-candidate drift reporting
- created the first shared corpus layout under `fixtures/analysis-corpus/`
  with explicit buckets for:
  - `synthetic/`
  - `manifests/`
  - `external-small/`
  - `external-large/`
- recorded the first cross-analyzer fixture taxonomy:
  - tonal
  - noise
  - pulse
  - sustained
  - loudness
  - silence
  - rate-policy
  - semantic
- kept licensing and artifact-size posture explicit in the corpus README rather
  than implying that large or externally sourced audio belongs in the repo
- added one repo-owned entry point for the harness through
  `effigy acceptance:analysis --repo .`
- updated roadmap and architecture docs to reflect that `007.1` is now opened
  and implemented

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis`
- `git diff --check`
- `effigy test --repo .`

Known limits after this tranche:

- the harness layer currently measures whole-buffer offline analyzer outputs
  only; there is no streaming or per-segment comparison surface yet
- threshold policy is still shape-only at the roadmap level; analyzer families
  do not yet have frozen practical metrics or drift bounds
- the shared corpus layout currently contains structure and metadata rules, not
  a richer set of committed reference audio assets

Next task:

Continue `g02.007` by freezing practical metric and drift policies for the
first analyzer families so the new harness can produce threshold-backed
acceptance evidence.
