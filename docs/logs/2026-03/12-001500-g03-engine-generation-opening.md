# g03 Engine Generation Opening

Time: 2026-03-12 00:15 Europe/London
Area: `roadmaps`
Status: complete

## Summary

Opened Signal `g03` as the next active repo-local roadmap generation.

The previous Signal continuation queue (`g02`) completed the reusable DSP and
analysis depth lane. The next strong sequencing boundary is engine-oriented
work inside Signal-owned crates rather than another analysis expansion pass.

`g03` now carries:

- routed mixer graph, buses, and routing-topology depth
- runtime metering, loudness, and diagnostics export
- automation playback and control-resolution depth
- tempo-map, warp, clip-processing, and render substrate
- plugin device-chain execution, latency compensation, and state recall
- offline render/freeze/stem export depth
- profiling, soak harnesses, and runtime fault hardening

## Why this boundary is separate

- The analysis-heavy `g02` queue is closed and coherent.
- The next work is foundation-first engine depth, not more descriptor breadth.
- Products downstream will benefit more from stronger reusable runtime,
  routing, render, and plugin-chain substrate than from another immediate
  analysis branch.

## Validation

- `git diff --check`

## Next Task

Start `g03.001` and make the routed mixer graph and bus topology explicit
enough to support metering, automation, warp, and render work on one stable
engine contract.
