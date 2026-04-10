# Signal Docs

Signal uses the Northstar documentation shape as a generic library-system
planning surface.

## Core Sections

- `vision/`
- `architecture/`
- `contracts/`
- `roadmaps/`
- `logs/`

## Optional Sections In Use

- `research/`
- `specs/` for the active lane-first strict `g09` surface

Signal is still using a baseline Northstar spine overall, but the active `g09`
queue now has a lane-first strict surface under `docs/specs/`.

## Current Entry Points

- Vision: [vision/001-signal-vision.md](./vision/001-signal-vision.md)
- Architecture: [architecture/system-architecture.md](./architecture/system-architecture.md)
- Product guardrails: [architecture/product-guardrails.md](./architecture/product-guardrails.md)
- Package map: [architecture/package-map.md](./architecture/package-map.md)
- DSP and analysis feature reference: [architecture/dsp-analysis-feature-reference.md](./architecture/dsp-analysis-feature-reference.md)
- Graph and runtime feature reference: [architecture/graph-runtime-feature-reference.md](./architecture/graph-runtime-feature-reference.md)
- Working rules: [contracts/001-working-rules.md](./contracts/001-working-rules.md)
- Shared DSP boundary: [contracts/001-shared-dsp-and-host-boundary.md](./contracts/001-shared-dsp-and-host-boundary.md)
- Supervisor export boundary: [contracts/002-supervisor-export-schema-and-report-boundary.md](./contracts/002-supervisor-export-schema-and-report-boundary.md)
- Active strict-lane spec: [specs/001-g09-lane-first-strict-adoption.md](./specs/001-g09-lane-first-strict-adoption.md)
- Active strict-lane card:
  [specs/batch-cards/034-g09-013-audit-closeout-proof-bundle.md](./specs/batch-cards/034-g09-013-audit-closeout-proof-bundle.md)
- Research index: [research/master-index.md](./research/master-index.md)

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Working Rule

- treat Signal docs as the canonical authority for reusable library/runtime
  building blocks
- keep Finch and Loophole wrapper notes outside Signal unless they affect the
  reusable library boundary
- keep section indexes aligned to Northstar conventions
- treat `legacy/cpp/` as reference surface, not primary implementation surface
- treat the active `g09` queue as a lane-first strict Northstar surface under
  `docs/specs/`
- in the strict lane, treat a bare `continue` as "follow the previous closeout's
  `Next Task`" rather than as permission to infer a new batch
- if there is no current ready card, re-enter planning instead of improvising

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/034-g09-013-audit-closeout-proof-bundle.md`.
