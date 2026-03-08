# Package Map and Runtime Entrypoint Naming

Date: 2026-03-08
Owner: core-product

## Summary

Defined the first broader naming proposal for Signal packages and host
entrypoints so the shared research corpus no longer depends on Finch-shaped
crate examples.

## Main decisions

- prefer `signal-<layer>` or `signal-<layer>-<domain>` naming
- prefer `signal-primitives` over `signal-core`
- prefer `signal-analysis-rhythm`, `signal-analysis-tonal`, and
  `signal-analysis-loudness` over narrower feature-first names
- prefer `signal-dsp-spectral` for FFT/STFT and spectral transforms
- freeze the first host names:
  - `signal-host-local`
  - `signal-host-server`
  - `signal-plugin-sandbox`

## Updated docs

- `docs/architecture/package-map.md`
- `docs/architecture/system-architecture.md`
- `docs/research/master-index.md`
- `docs/research/source-hubs/001-rust-audio-ecosystem.md`
- `docs/research/source-hubs/002-signal-library-architecture.md`
- `docs/research/value-tracks/*`
- `docs/research/algorithm-specs/*`
- `docs/research/specimen-dossiers/essentia.md`
- `docs/roadmaps/g01/002-package-map-and-runtime-entrypoint-naming.md`

## Validation

- manual doc review
- `git diff --check`

## Next Task

Create the first actual Signal workspace directories and package manifests for
the frozen names, starting with `signal-primitives`, `signal-dsp`,
`signal-dsp-spectral`, `signal-analysis`, `signal-analysis-rhythm`,
`signal-analysis-tonal`, `signal-analysis-loudness`, `signal-graph`, and
`signal-runtime`.
