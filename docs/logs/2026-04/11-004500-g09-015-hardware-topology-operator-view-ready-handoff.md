# g09.015 - Hardware Topology Operator View Ready Handoff

Date: 2026-04-11  
Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`  
Ready card: `docs/specs/batch-cards/057-g09-015-hardware-topology-operator-view.md`

## Why This Follow-On Is Ready

The plugin, analysis, graph, DSP, and runtime families now have rendered
operator companions. The next remaining receipt-heavy demo family is hardware
topology diagnostics.

This is the next honest seam because:

- `signal.demo.hardware.topology-diagnostics` already captures bounded native
  local-host CoreAudio posture and simulated server-host Linux backend posture
- the gap is presentation-only rather than backend or device behavior
- the work can stay browser-native and low-dependency, consistent with the
  active `g09.015` contract

## Batch Boundary

- add a rendered operator companion for the hardware topology diagnostics demo
- keep native-versus-simulated posture explicit
- align manifest, operator notes, receipt, and coverage notes to the rendered
  view
- do not widen into device control, host redesign, or native Linux backend
  implementation work

## Validation For This Planning Step

- `effigy tasks`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/057-g09-015-hardware-topology-operator-view.md`.
