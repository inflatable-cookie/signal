# g09.015 - Plugin Sandbox Lifecycle Operator View Closeout

Date: 2026-04-11  
Card: `docs/roadmaps/g09/batch-cards/059-g09-015-plugin-sandbox-lifecycle-operator-view.md`  
Status: complete

## Summary

Promoted the sandbox lifecycle demo from receipt-only output into a rendered
browser-native operator companion. The dedicated sandbox/broker lifecycle proof
surface is now visually inspectable across ready, attach, healthy run, timeout
run, teardown, and shutdown posture without widening into a broker console.

## Delivered

- added a rendered companion view to
  `demos/scripts/run_sandbox_lifecycle_demo.py`
- generated `demos/receipts/plugin-sandbox-lifecycle.view.html`
- aligned the manifest and operator notes in:
  - `demos/manifests/plugin-sandbox-lifecycle.demo.json`
  - `demos/scenarios/plugin-sandbox-lifecycle.default.md`
- recorded the rendered companion in
  `demos/receipts/plugin-sandbox-lifecycle.receipt.json`

## Validation Run

- `python3 demos/scripts/run_sandbox_lifecycle_demo.py`
- `effigy demo:sandbox-lifecycle`
- `effigy qa:docs`
- `effigy qa:northstar`

## Result

- broker lifecycle and timeout recovery posture are now visually inspectable
  without reading raw JSON first
- the remaining receipt-only surfaces are now the platform boundary demos
- `g09.015` remains active and advances directly into `060`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/060-g09-015-platform-boundary-operator-views.md`.
