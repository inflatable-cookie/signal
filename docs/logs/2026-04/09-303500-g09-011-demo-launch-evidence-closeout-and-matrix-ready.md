# 2026-04-09 - g09.011 Demo Launch Evidence Closeout And Matrix Ready

## Summary

Closed strict card `020-g09-011-demo-launch-and-evidence-conventions` after
freezing the shared launch-command posture and evidence-capture conventions,
then promoted the coverage-matrix seam as the next `g09.011` batch.

## Implementation

- extended the demo manifest schema with an explicit `evidence` block
- added a demo receipt template and operator-notes template under
  `demos/templates/`
- added `demos/receipts/` as the shared home for machine-readable live demo
  receipts
- documented launch-command posture and evidence conventions in `demos/README.md`
- added `effigy demo:conventions` as the shared validation task for the
  conventions pack

## Validation

- `effigy demo:conventions`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

- full domain demos and the crate-to-demo coverage matrix remain deferred to
  later milestones
- the next clean seam is the explicit coverage matrix, not domain demo
  implementation

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/021-g09-011-demo-coverage-matrix.md`.
