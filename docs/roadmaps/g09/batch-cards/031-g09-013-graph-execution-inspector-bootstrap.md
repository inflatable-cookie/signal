# 031 - g09.013 Graph Execution Inspector Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-10
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.013
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md, docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md, docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md, docs/contracts/036-spatial-adapter-execution-contract.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/013-dsp-graph-analysis-interactive-demo-suite-and-audit-closeout-proof.md
Auto-start next card: no

## Objective

Start `g09.013` with the first honest DSP/graph demo seam by turning the
existing multichannel, sidechain, multi-bus, and spatial graph boundary
surfaces into one repo-owned graph execution inspector scenario.

## Scope

- stay inside the current `signal-supervisor-tools` graph-routing descriptor
  commands and the existing `effigy acceptance:multichannel-boundary`,
  `effigy acceptance:sidechain-boundary`,
  `effigy acceptance:multi-bus-boundary`, and
  `effigy acceptance:spatial-boundary` tasks
- build one repo-owned manifest, operator notes file, launch task, and receipt
  under `demos/`
- keep the surface focused on graph execution meaning and do not widen into a
  DAW workflow shell, generalized media browser, or tutorial UI
- promote `signal-primitives` and `signal-graph` to live coverage only if the
  manifest and receipt are actually in place
- treat runtime and supervisor crates as reused proof transport, not new
  ownership claims for this batch

## Steps

1. Freeze the bounded graph seam from contracts `032`, `033`, `034`, `036`,
   and `079`.
2. Add a graph execution inspector manifest, operator notes, receipt path, and
   Effigy launch task under `demos/`.
3. Implement a repo-owned wrapper that runs the current graph-routing boundary
   descriptor commands plus the existing acceptance lanes, then emits one
   machine-readable receipt.
4. Keep the surface explicit about what it proves: multichannel layout,
   sidechain routing, multi-bus topology, and spatial execution meaning from
   one graph-focused inspection family.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures the current graph-routing boundary
  family without flattening it into a generic product demo
- the receipt keeps graph execution meaning explicit across multichannel,
  sidechain, multi-bus, and spatial seams
- the batch stays bounded to the graph execution inspector bootstrap seam
- focused validation passes

## Evidence Required

- batch log for the next `g09.013` tranche
- validation actually run
- explicit note which DSP and analysis demo seams remain deferred after this
  batch

## Outcome

- added the live graph execution inspector surface under `demos/` with one
  repo-owned manifest, scenario notes file, launch script, Effigy task, and
  machine-readable receipt
- promoted `signal-primitives` and `signal-graph` to live demo coverage in the
  coverage matrix
- repaired stale graph-routing acceptance wiring so the frozen multichannel,
  sidechain, multi-bus, and spatial boundary family now executes cleanly
  through the shared demo wrapper
- left the next `g09.013` seam unpromoted because DSP processing-lab versus
  analysis feature-inspector still needs fresh planning judgment

## Stop Conditions

- the work starts redesigning graph/runtime behavior instead of wrapping the
  current descriptor and acceptance surfaces
- the seam needs fresh planning about DSP laboratory or analysis corpus posture
  before the demo can be executed honestly
- the batch starts implementing a product shell, visual graph editor, or
  generalized tutorial UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.013` seam is DSP processing-lab bootstrap, analysis feature
inspector bootstrap, or a continued planning pause.
