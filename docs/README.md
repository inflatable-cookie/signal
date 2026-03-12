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

Signal does not need top-level `schemas/`, `diagrams/`, or `specs/` right now,
so they are intentionally absent.

## Current Entry Points

- Vision: [vision/001-signal-vision.md](./vision/001-signal-vision.md)
- Architecture: [architecture/system-architecture.md](./architecture/system-architecture.md)
- Package map: [architecture/package-map.md](./architecture/package-map.md)
- DSP and analysis feature reference: [architecture/dsp-analysis-feature-reference.md](./architecture/dsp-analysis-feature-reference.md)
- Graph and runtime feature reference: [architecture/graph-runtime-feature-reference.md](./architecture/graph-runtime-feature-reference.md)
- Shared DSP boundary: [contracts/001-shared-dsp-and-host-boundary.md](./contracts/001-shared-dsp-and-host-boundary.md)
- Supervisor export boundary: [contracts/002-supervisor-export-schema-and-report-boundary.md](./contracts/002-supervisor-export-schema-and-report-boundary.md)
- Research index: [research/master-index.md](./research/master-index.md)

## Working Rule

- treat Signal docs as the canonical authority for reusable library/runtime
  building blocks
- keep Finch and Loophole wrapper notes outside Signal unless they affect the
  reusable library boundary
- keep section indexes aligned to Northstar conventions
- treat `legacy/cpp/` as reference surface, not primary implementation surface

## Next Task

Reorient the remaining Signal docs around the generic library posture and move
legacy C++ implementation material behind a clear reference boundary.
