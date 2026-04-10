# Runtime Supervisor Boundary Companion

Status: active
Updated: 2026-04-10

## Scenario

- manifest id: `signal.demo.runtime.supervisor-boundary-companion`
- scenario id: `signal.demo.runtime.supervisor-boundary-companion.default`

## Launch

- command: `effigy demo:supervisor-runtime-boundary-companion`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt captures at least the interruption and fault-diagnostic
  boundary descriptors through the current `signal-supervisor-tools` CLI
- confirm the receipt keeps the acceptance-task and contract-path truth from
  those descriptors visible instead of reducing them to a pass/fail flag
- confirm the receipt explicitly says it complements the existing runtime
  recovery inspector rather than replacing the example-backed runtime demo

## Environment Notes

- no external plugin bundle, host process, or hardware device prerequisite is
  required
- this surface intentionally uses the current `signal-supervisor-tools`
  descriptor commands directly instead of inventing a new supervisor binary mode

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/runtime-supervisor-boundary-companion.receipt.json`
- plugin capability browsing remains deferred and should stay explicit rather
  than being implied by this runtime-family companion surface
