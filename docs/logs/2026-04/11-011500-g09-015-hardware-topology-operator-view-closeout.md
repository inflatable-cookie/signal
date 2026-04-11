# g09.015 - Hardware Topology Operator View Closeout

Date: 2026-04-11  
Card: `docs/specs/batch-cards/057-g09-015-hardware-topology-operator-view.md`  
Status: complete

## Summary

Promoted the hardware topology diagnostics demo from receipt-only output into a
rendered browser-native operator companion. The hardware family now has a
visually inspectable native-versus-simulated posture surface without widening
into a hardware control shell or new backend behavior.

## Delivered

- added a rendered companion view to
  `demos/scripts/run_hardware_topology_diagnostics_demo.py`
- generated `demos/receipts/hardware-topology-diagnostics.view.html`
- aligned the manifest and operator notes in:
  - `demos/manifests/hardware-topology-diagnostics.demo.json`
  - `demos/scenarios/hardware-topology-diagnostics.default.md`
- recorded the rendered companion and bounded host-capture truth in
  `demos/receipts/hardware-topology-diagnostics.receipt.json`
- added bounded host summary capture so the demo can accept a valid summary
  line without waiting indefinitely for child process exit
- refreshed the active roadmap and front-door/currentness surfaces to show
  there is no ready `g09.015` card now

## Validation Run

- `python3 demos/scripts/run_hardware_topology_diagnostics_demo.py`
- `effigy demo:hardware-topology-diagnostics`
- `effigy qa:docs`
- `effigy qa:northstar`

## Result

- native CoreAudio posture and simulated Linux backend posture are now visually
  inspectable without reading raw JSON first
- the hardware family is no longer receipt-only at the operator layer
- `g09.015` remains active, but there is currently no ready card

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
