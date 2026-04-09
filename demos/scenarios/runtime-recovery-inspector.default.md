# Runtime Recovery Inspector Operator Notes

Status: active
Updated: 2026-04-09

## Scenario

- manifest id: `signal.demo.runtime.recovery-inspector`
- scenario id: `signal.demo.runtime.recovery-inspector.default`

## Launch

- command: `effigy demo:runtime-recovery-inspector`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt captures readiness, watchdog, and plugin-fault truth from
  the runtime supervisor report
- confirm the runtime report keeps safe-mode posture explicit even while fault
  history is present
- confirm degraded hardware/backend surfaces remain inspectable instead of
  disappearing from the report

## Environment Notes

- no external plugin bundle, host process, or hardware device prerequisite is
  required
- this bootstrap surface intentionally uses the existing
  `signal-runtime --example supervisor_report_demo` path directly

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/runtime-recovery-inspector.receipt.json`
- this scenario does not yet claim host comparison, plugin capability browsing,
  or hardware live demo coverage
