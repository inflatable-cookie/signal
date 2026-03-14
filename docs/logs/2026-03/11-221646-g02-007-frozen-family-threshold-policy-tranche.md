# 2026-03-11 22:16:46 GMT - g02.007 frozen family threshold policy tranche

Continued `g02.007` by freezing the first practical acceptance and drift
policies for the shared analysis corpus across three analyzer families:
character descriptors, loudness, and semantic inference.

This tranche matters because the new corpus and harness surface now protects
real analyzer outputs with repo-owned thresholds instead of stopping at
structure, naming, and helper APIs.

Implemented changes:

- added first frozen policy manifest at
  `fixtures/analysis-corpus/manifests/frozen-family-policies-v1.md`
- documented practical acceptance metrics and drift posture for:
  - `signal-analysis-character`
  - `signal-analysis-loudness`
  - `signal-analysis-embed`
- froze confidence handling explicitly:
  - bounded descriptor and semantic confidence thresholds where those surfaces
    already exist
  - performance kept report-only through harness `elapsed_ms` rather than
    hard-failed at this corpus size
- promoted `effigy acceptance:analysis` into a real multi-family
  sequence covering:
  - shared harness smoke coverage in `signal-analysis`
  - character acceptance cases
  - loudness acceptance cases
  - semantic acceptance cases
- added shared-harness acceptance coverage in:
  - `crates/signal-analysis-character/src/lib.rs`
  - `crates/signal-analysis-loudness/src/lib.rs`
  - `crates/signal-analysis-embed/src/lib.rs`
- froze the first canonical synthetic cases:
  - `character:tone:sine440`
  - `character:noise:deterministic`
  - `character:pulse:adsr`
  - `loudness:quiet-sine`
  - `loudness:loud-sine`
  - `loudness:level-step`
  - `semantic:tone:sine440`
  - `semantic:noise:deterministic`
  - `semantic:pulse:adsr`
- recorded the current residual gap directly in the manifest:
  - `signal-analysis-rhythm` and `signal-analysis-tonal` are still deferred
    because their ambiguity-rich outputs need a more explicit policy surface

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis-character`
- `cargo test -p signal-analysis-loudness`
- `cargo test -p signal-analysis-embed`
- `effigy acceptance:analysis`
- `git diff --check`
- `effigy test`

Known limits after this tranche:

- frozen policies currently cover character, loudness, and semantic families
  only
- rhythm and tonal still need ambiguity-aware acceptance and drift policy work
- performance remains recorded but not thresholded; that is intentional at the
  current corpus size

Next task:

Log acceptance evidence for the frozen families and add explicit residual-risk
notes for rhythm and tonal policies before broadening the corpus or closing
`g02.007`.
