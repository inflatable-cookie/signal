# 2026-03-11 22:45:00 GMT - g02.006 descriptor embedding baseline opening tranche

Opened `g02.006` by adding the first reusable Signal-owned embedding and
semantic-analysis crate on top of the shared character descriptor packs.

This batch matters because Signal now owns a host-neutral inference boundary
instead of forcing semantic or catalog inference logic back into app-local
repositories.

Implemented changes:

- added `crates/signal-analysis-embed` as the first embedding/inference crate
- implemented `SemanticEmbedder` and `SemanticEmbedderConfig` so callers can:
  - resolve a requested model id
  - control fail-closed versus built-in fallback behavior
  - reuse the shared `CharacterAnalyzerConfig`
  - cap ranked semantic tag output
- defined explicit model contracts through:
  - `SemanticModelSpec`
  - `SemanticModelVersion`
  - `SemanticModelResourceProfile`
  - `ModelLoadError`
- shipped the first built-in deterministic model:
  - model id: `signal:descriptor-embed:v1`
  - source: built-in
  - dimensions: `8`
  - network requirement: none
  - behavior: descriptor-pack projection rather than external weights
- implemented one practical inference path that projects shared descriptor packs
  into:
  - an 8-dimensional embedding
  - ranked semantic tags for `TonalFocus`, `TexturalNoise`, `PulseDriven`,
    `SustainedBody`, and `DynamicPunch`
  - confidence and margin diagnostics
- kept descriptor-pack integration explicit by preserving the full
  `CharacterAnalysisResult` inside `SemanticAnalysisResult`
- updated the DSP/analysis feature reference and roadmap state to reflect the
  new crate boundary and active `g02.006` posture

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis-embed`

Remaining limits after this tranche:

- only the built-in deterministic descriptor model is available; no external
  weight loading or ONNX path exists yet
- semantic tags are fixed and intentionally small; there is no taxonomy-mapping
  or nearest-neighbor retrieval layer yet
- closeout still needs explicit semantic examples and final portability notes

Next task:

Close `g02.006` by recording remaining inference gaps, portability assumptions,
and baseline semantic examples before advancing to `g02.007`.
