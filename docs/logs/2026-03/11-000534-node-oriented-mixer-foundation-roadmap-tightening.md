# Node-Oriented Mixer Foundation Roadmap Tightening

Status: active
Owner: core-product
Updated: 2026-03-11

## Summary

Tightened the active Signal roadmap surface so node-oriented mixer concepts stay
visible through the foundation milestones instead of being treated as a late
topology garnish.

## What Changed

- updated `docs/roadmaps/g01/README.md` so the active sequencing rules
  explicitly keep console-node, track-lane, bus, and routing intent visible in
  early engine decisions
- expanded `g01.006` to require graph-contract compatibility with future
  console-node, track-lane, send, and return semantics
- expanded `g01.007` so runtime scheduling and diagnostics preserve
  node-oriented topology rather than flattening it into host-local policy
- expanded `g01.008` so real host/device execution proves at least one credible
  node/lane/bus path instead of only a flattened output callback
- expanded `g01.009` so plugin-backed nodes are planned as first-class members
  of the same node-oriented mixer graph as native Signal processing

## Why

Signal's differentiators will be easier to implement if the graph, runtime,
host, and plugin baselines already assume a node-oriented mixer future. Making
that explicit now reduces the risk of later retrofits around console nodes,
track lanes, buses, sends, and returns.

## Validation

- `git diff --check`

## Next Task

Keep `g01.006` through `g01.009` implementation work honest against these
roadmap constraints, especially whenever graph-node metadata, execution lanes,
or host validation fixtures are introduced.
