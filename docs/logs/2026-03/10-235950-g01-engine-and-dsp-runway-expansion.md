# Signal Log: g01 Engine and DSP Runway Expansion

Date: 2026-03-10
Status: complete
Owner: core-product

## Summary

Expanded the active Signal roadmap surface so a parallel implementation thread
can work from a dependency-first engine and DSP runway instead of inventing its
own next milestones.

## Changes

1. rewrote `docs/roadmaps/g01/README.md` as a real generation guide rather than
   a four-file list
2. added detailed queued milestones:
   - `g01.005` core DSP kernel and control-signal baseline
   - `g01.006` executable graph routing, latency, and parameter application
   - `g01.007` runtime transport, scheduler, and engine processing baseline
   - `g01.008` device-backed host audio I/O and diagnostics baseline
   - `g01.009` plugin hosting, sandbox processing, and graph-node baseline
3. updated the top-level roadmap entry points so the new engine/DSP runway is
   visible from the roadmaps front door and generation index

## Validation

- `git diff --check`
- `effigy validate --repo .`

## Risks

- The currently active `g01.004` milestone is still the immediate dependency
  gate, so the next Signal thread should treat `g01.005` as the next major
  implementation contract rather than assuming every trust-edge concern is
  already closed.
- These new milestones are detailed enough to guide work, but they are still
  roadmap contracts rather than exhaustive task backlogs for every sub-batch.

## Next Task

Open `g01.005` and start landing the reusable DSP/control kernel layer that the
rest of the engine, host, and plugin milestones now depend on.
