# Graph Execution Inspector

Status: active
Updated: 2026-04-10

## Scenario

- manifest id: `signal.demo.graph.execution-inspector`
- scenario id: `signal.demo.graph.execution-inspector.default`

## Launch

- command: `effigy demo:graph-execution-inspector`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt captures the current
  `--describe-multichannel-boundary --format=json`,
  `--describe-sidechain-boundary --format=json`,
  `--describe-multi-bus-boundary --format=json`, and
  `--describe-spatial-boundary --format=json` descriptor outputs
- confirm the receipt records that the existing multichannel, sidechain,
  multi-bus, and spatial acceptance lanes completed
- confirm the receipt keeps graph execution meaning explicit rather than
  pretending to be a DAW shell or generalized product demo

## Environment Notes

- this surface depends on the current `signal-supervisor-tools` graph-routing
  boundary descriptors and their existing acceptance lanes
- it does not require a new graph editor, custom visualizer, or tutorial UI
- runtime and supervisor crates are reused proof transport here, not new
  ownership claims for this graph batch

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/graph-execution-inspector.receipt.json`
- DSP processing-lab and analysis feature-inspector work remain deferred and
  should stay explicit rather than being implied by this graph-focused surface
