# Signal Docs

These docs use the Northstar structure and are the canonical planning and
research authority for Signal.

## Sections

- `vision/`: long-horizon outcomes, strategic constraints, and target envelopes
- `architecture/`: shared runtime shape, crate boundaries, and invariants
- `contracts/`: explicit reusable-DSP and host-boundary rules
- `roadmaps/`: active execution queue and backlog
- `research/`: DSP, analysis, algorithm, and dependency research authority
- `logs/`: batch-level evidence and decision records

## Current entry points

- Vision: [`vision/001-signal-vision.md`](./vision/001-signal-vision.md)
- Architecture: [`architecture/system-architecture.md`](./architecture/system-architecture.md)
- Package map: [`architecture/package-map.md`](./architecture/package-map.md)
- Core boundary: [`contracts/001-shared-dsp-and-host-boundary.md`](./contracts/001-shared-dsp-and-host-boundary.md)
- Supervisor export: [`contracts/002-supervisor-export-schema-and-report-boundary.md`](./contracts/002-supervisor-export-schema-and-report-boundary.md)
- Research index: [`research/master-index.md`](./research/master-index.md)
- Active roadmap: [`roadmaps/g01/004-trust-edge-package-shell-expansion.md`](./roadmaps/g01/004-trust-edge-package-shell-expansion.md)

## Working rules

- Treat these docs as the authority for shared DSP and analysis planning.
- Keep reusable DSP and algorithm research here, not in Finch or Loophole-local
  wrapper docs.
- Let Finch keep only wrapper/integration notes and migration breadcrumbs back
  to Signal.
- Log meaningful batches, not tiny edits.

## Next task

Map the new trust-edge package shells onto real runtime-host and sandbox modules
so the Signal workspace boundary is visible in code as well as docs.
