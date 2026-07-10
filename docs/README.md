# Signal Docs

Signal uses the Northstar documentation shape as the repo-owned authority layer
for the reusable library/runtime surface.

## Core Sections

- `vision/`
- `architecture/`
- `contracts/`
- `roadmaps/`
- `logs/`

## Optional Sections In Use

- `research/`
- `specs/` for closed strict-lane references and any future reopened strict
  lane

Signal is back in a baseline Northstar posture. There is currently no active
strict lane.

## Current Entry Points

- Vision: [vision/001-signal-vision.md](./vision/001-signal-vision.md)
- Architecture: [architecture/system-architecture.md](./architecture/system-architecture.md)
- Product guardrails: [architecture/product-guardrails.md](./architecture/product-guardrails.md)
- Package map: [architecture/package-map.md](./architecture/package-map.md)
- DSP and analysis feature reference: [architecture/dsp-analysis-feature-reference.md](./architecture/dsp-analysis-feature-reference.md)
- Offline time-stretch synthesis: [architecture/offline-time-stretch-synthesis.md](./architecture/offline-time-stretch-synthesis.md)
- Graph and runtime feature reference: [architecture/graph-runtime-feature-reference.md](./architecture/graph-runtime-feature-reference.md)
- Working rules: [contracts/001-working-rules.md](./contracts/001-working-rules.md)
- Shared DSP boundary: [contracts/001-shared-dsp-and-host-boundary.md](./contracts/001-shared-dsp-and-host-boundary.md)
- Offline stretch synthesis policy: [contracts/082-offline-time-stretch-synthesis-policy-contract.md](./contracts/082-offline-time-stretch-synthesis-policy-contract.md)
- Supervisor export boundary: [contracts/002-supervisor-export-schema-and-report-boundary.md](./contracts/002-supervisor-export-schema-and-report-boundary.md)
- Roadmap index: [roadmaps/README.md](./roadmaps/README.md)
- Generation index: [roadmaps/generation-index.md](./roadmaps/generation-index.md)
- Active roadmap: [roadmaps/g10/029-stretch-correctness-and-listening-gate.md](./roadmaps/g10/029-stretch-correctness-and-listening-gate.md)
- Strict-lane reference: [specs/001-g09-lane-first-strict-adoption.md](./specs/001-g09-lane-first-strict-adoption.md)
- Active strict-lane card: none
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
- treat an active generation as a lane-first strict Northstar surface under
  `docs/specs/` only while that generation is explicitly open
- if there is no active strict lane, use the roadmap and contract front doors
  instead of reading old batch-card state as current authority
- in the strict lane, treat a bare `continue` as "follow the previous closeout's
  `Next Task`" rather than as permission to infer a new batch

## Next Task

Use this front door to find the current authority surfaces first:
`vision/`, `architecture/`, `contracts/`, and `roadmaps/`. Only drop into
`specs/` when a new strict lane is explicitly reopened.
