# Operator Notes Template

Status: template
Updated: 2026-04-09

## Scenario

- manifest id: `signal.demo.shared.program-shape`
- scenario id: `signal.demo.shared.program-shape.default`

## Launch

- command: `cargo run -p signal-runtime --example supervisor_report_demo`
- owner surface: `cargo-example`

## Expected Human Checks

- confirm the command launches the declared repo-owned surface
- confirm the scenario identity matches the manifest
- confirm any future receipt path is updated after a real run

## Environment Notes

- list platform or device prerequisites here when a real demo surface exists

## Evidence Notes

- record human observations here only when they are not representable in the
  machine-readable receipt

