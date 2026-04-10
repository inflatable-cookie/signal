# Demo Program

Status: active
Updated: 2026-04-09

## Purpose

This folder is the shared substrate for repo-owned Signal demo programs.

It does not replace crate code. It defines the authority layer that turns demo
commands into inspectable proof instead of loose examples.

## Layout

- `manifest.schema.json`
  - machine-readable schema for every official demo manifest
- `templates/`
  - manifest examples and starter shapes for new demo surfaces
- `manifests/`
  - machine-readable manifests for official demo binaries or scenario bundles
- `coverage-matrix.json`
  - machine-readable crate-to-demo coverage inventory for the active workspace
- `coverage-matrix.md`
  - human-readable view of the same coverage inventory
- `receipts/`
  - machine-readable run receipts for official demos
- `scenarios/`
  - future operator-facing notes for shared scenario bundles when human checks
    matter

## Program Shape

- the authoritative declaration for an official demo is a manifest in
  `demos/manifests/`
- a manifest may point at:
  - `cargo run -p <crate> --example <name>`
  - `cargo run -p <crate> --bin <name>`
  - a future repo-owned wrapper command exposed through Effigy
- existing crate `examples/` are not official demo surfaces until a manifest
  claims them
- scenario identity belongs in the manifest, not only in code comments or
  operator memory
- official active demos should prefer an `effigy-task` launch owner surface once
  they are promoted beyond placeholder or template status
- cargo example or cargo bin launch commands are acceptable for planned
  substrate and bootstrap stages while the later launch posture is still being
  installed

## Naming Rules

- manifest files use kebab-case:
  `<domain>-<surface>.demo.json`
- manifest ids use the stable namespace:
  `signal.demo.<domain>.<surface>`
- scenario ids use the stable namespace:
  `signal.demo.<domain>.<surface>.<scenario>`

## Grouping Rule

- prefer a shared domain demo when one operator workflow naturally proves
  multiple closely related crates together
- use a dedicated demo only when a crate has a distinct operator workflow,
  unique external prerequisites, or would be hidden by a shared surface

## Evidence Convention

- every official demo manifest must name:
  - a machine-readable receipt path
  - an operator-notes path
  - a shared capture mode
- receipts belong in `demos/receipts/` for live demos or `demos/templates/`
  for substrate examples
- operator notes belong in `demos/scenarios/` for live demos or
  `demos/templates/` for substrate examples
- receipts should capture the launched command, scenario identity, run status,
  and explicit operator-check outcomes
- operator notes should capture the human checks that are meaningful but not
  machine-readable

## Launch Convention

- official demo launches must be repo-owned and explicit in the manifest
- preferred launch owner surface:
  - `effigy-task`
- temporary bootstrap owner surfaces:
  - `cargo-example`
  - `cargo-bin`
- a later demo batch should promote active demos from cargo-owned launch
  commands to Effigy-owned launch tasks as the substrate matures
- the first official live demo surfaces are:
  - `signal.demo.plugin.sandbox-lifecycle`
  - `signal.demo.runtime.recovery-inspector`
  - `signal.demo.runtime.supervisor-boundary-companion`
  - `signal.demo.host.local-server-compare`
  - `signal.demo.hardware.topology-diagnostics`
  - `signal.demo.graph.execution-inspector`
  - `signal.demo.dsp.processing-lab`

## Current Boundary

- this substrate pack freezes the program shape, launch/evidence conventions,
  coverage matrix, and first official live demo manifest
- broader plugin, host, runtime, hardware, DSP, graph, and analysis demo
  breadth remains deferred to later `g09.012+` and `g09.013+` batches

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.013` seam is analysis feature-inspector bootstrap or a continued
planning pause.
