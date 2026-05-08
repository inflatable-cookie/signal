# 2026-04-09 - g09.011 Demo Program Shape Closeout And Launch Ready

## Summary

Closed strict card `019-g09-011-demo-program-shape` after freezing the shared
demo substrate location, manifest schema, and grouping rule, then promoted the
next `g09.011` seam for launch and evidence conventions.

## Implementation

- added workspace-root `demos/` as the shared demo-substrate authority layer
- added `demos/manifest.schema.json` for official demo manifests
- added `demos/templates/demo-manifest.example.json` as the starter shape for
  future official demo manifests
- added `demos/manifests/` and `demos/scenarios/` placeholders
- added `effigy demo:program-shape` as the first repo-owned task for validating
  the shared substrate shape
- recorded that existing crate examples are not official demos until a manifest
  claims them

## Validation

- `effigy demo:program-shape`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

- domain demos and crate-to-demo coverage matrix remain deferred to later
  milestones
- the next clean seam is shared launch and evidence conventions, not domain
  demo breadth

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/020-g09-011-demo-launch-and-evidence-conventions.md`.
