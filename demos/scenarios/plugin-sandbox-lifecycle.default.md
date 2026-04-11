# Sandbox Lifecycle Demo Operator Notes

Status: active
Updated: 2026-04-09

## Scenario

- manifest id: `signal.demo.plugin.sandbox-lifecycle`
- scenario id: `signal.demo.plugin.sandbox-lifecycle.default`

## Launch

- command: `effigy demo:sandbox-lifecycle`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt shows `ready`, `attached`, `running`, `timed_out`,
  `teardown_complete`, and `shutdown`
- confirm the explicit attach/status/teardown path is present in the same run,
  not only the one-shot demo path
- confirm the timeout path reports cleanup after interruption instead of
  crashing the broker
- confirm the rendered companion view at
  `demos/receipts/plugin-sandbox-lifecycle.view.html` makes lifecycle and
  timeout recovery posture visually inspectable without reading the raw receipt
  first

## Environment Notes

- no external plugin bundle, host, or device prerequisite is required
- this bootstrap surface intentionally uses the existing
  `signal-plugin-sandbox` broker binary directly

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/plugin-sandbox-lifecycle.receipt.json`
- this scenario does not yet claim plugin capability browsing or format-specific
  live demo coverage
- the rendered companion view is presentation-only over the existing broker
  transcript lines and does not claim an interactive broker console
