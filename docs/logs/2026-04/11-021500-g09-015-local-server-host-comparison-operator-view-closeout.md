# g09.015 - Local Server Host Comparison Operator View Closeout

Date: 2026-04-11  
Card: `docs/roadmaps/g09/batch-cards/058-g09-015-local-server-host-comparison-operator-view.md`  
Status: complete

## Summary

Promoted the local-versus-server host comparison demo from receipt-only output
into a rendered browser-native operator companion. The host family now has a
visually inspectable comparison surface for shared lifecycle truth and the key
local-versus-server differences, without widening into a host UI shell.

## Delivered

- added a rendered companion view to
  `demos/scripts/run_local_server_host_comparison_demo.py`
- generated `demos/receipts/local-server-host-comparison.view.html`
- aligned the manifest and operator notes in:
  - `demos/manifests/local-server-host-comparison.demo.json`
  - `demos/scenarios/local-server-host-comparison.default.md`
- recorded the rendered companion in
  `demos/receipts/local-server-host-comparison.receipt.json`
- repaired the inherited readiness check in the comparison script so it reads
  the real summary truth instead of incorrectly marking both hosts not ready
- refreshed the active roadmap and front-door/currentness surfaces to show
  there is no ready `g09.015` card now

## Validation Run

- `python3 demos/scripts/run_local_server_host_comparison_demo.py`
- `effigy demo:local-server-host-comparison`
- `effigy qa:docs`
- `effigy qa:northstar`

## Result

- shared readiness, running posture, sandbox truth, and real local-versus-server
  differences are now visually inspectable without reading raw JSON first
- the host family is no longer receipt-only at the operator layer
- `g09.015` remains active, but there is currently no ready card

## Next Task

Re-enter planning for the active strict `g09` lane and leave a bounded runway
for the next `g09.015` operator-visible batches, including the next planning
checkpoint.
